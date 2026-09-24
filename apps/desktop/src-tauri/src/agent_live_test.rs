//! Phase 3's exit condition, run for real: "Agent can implement and verify a simple
//! repository task" (PRD §3587, docs/ROADMAP.md).
//!
//! This drives the production `run_task` — the same planner, state machine, permission
//! gate, audit log and verdict the app uses — against a real local model and a real git
//! repository on disk. Only the webview is replaced, by Tauri's mock runtime; the test
//! plays the user, approving each prompt the way "Allow once" would and printing
//! exactly what it approved.
//!
//! It does not take the runtime's word for success: after the task reports its verdict,
//! the test runs the repository's suite itself.
//!
//! Ignored by default: it needs a running Ollama with a tool-capable model, and on a
//! laptop CPU it takes minutes.
//!
//! ```sh
//! ANYCODE_LIVE_MODEL=qwen2.5:3b cargo test --lib agent_live -- --ignored --nocapture
//! ```
//!
//! `ANYCODE_LIVE_PROVIDER=openai_compatible` drives the same task through the OpenAI
//! adapter, pointed at Ollama's OpenAI-compatible API — exercising its streamed tool-call
//! reassembly against a real server.

use crate::agent_commands::{respond_to_approval, run_task, ApprovalResponse};
use crate::workspace::WorkspaceState;
use crate::AppState;
use anycode_store::Store;
use anycode_tools::ToolRegistry;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{mpsc, Mutex};
use std::time::{Duration, Instant};
use tauri::{Listener, Manager};
use uuid::Uuid;

const CALC: &str = "def add(a, b):
    return a + b


def multiply(a, b):
    raise NotImplementedError(\"multiply is not implemented yet\")
";

const TESTS: &str = "import unittest

from calc import add, multiply


class CalcTest(unittest.TestCase):
    def test_add(self):
        self.assertEqual(add(2, 3), 5)

    def test_multiply(self):
        self.assertEqual(multiply(4, 5), 20)
        self.assertEqual(multiply(-2, 3), -6)
        self.assertEqual(multiply(7, 0), 0)


if __name__ == \"__main__\":
    unittest.main()
";

const INSTRUCTION: &str = "Implement the `multiply` function in calc.py so that the test \
    suite passes. The tests run with `python3 -m unittest`.";

/// Channels the test watches, in the order a reader would want them explained.
const CHANNELS: [&str; 10] = [
    "state",
    "context",
    "plan",
    "replan",
    "tool_call",
    "approval_requested",
    "tool_result",
    "done",
    "error",
    "cancelled",
];

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args([
            "-c",
            "user.name=Any Code test",
            "-c",
            "user.email=test@anycode.invalid",
        ])
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git should run");
    assert!(status.success(), "git {args:?} failed");
}

/// A committed repository with one unimplemented function and a suite that fails.
fn make_repo() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("anycode-live-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("calc.py"), CALC).unwrap();
    std::fs::write(dir.join("test_calc.py"), TESTS).unwrap();
    std::fs::write(
        dir.join("README.md"),
        "# calc\n\nRun the tests with `python3 -m unittest`.\n",
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "."]);
    git(
        &dir,
        &["commit", "-q", "-m", "calc with an unimplemented multiply"],
    );
    dir
}

fn suite_passes(dir: &Path) -> bool {
    Command::new("python3")
        .args(["-m", "unittest", "-q"])
        .current_dir(dir)
        // The harness's own runs must not leave __pycache__ in the repo the agent inspects.
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// The suite as it was committed, run against whatever the agent left in calc.py. An
/// agent that "passed" by weakening test_calc.py passes its own copy, not this one.
fn original_suite_passes(dir: &Path) -> bool {
    let copy = dir.join("anycode_original_tests.py");
    std::fs::write(&copy, TESTS).unwrap();
    let passed = Command::new("python3")
        .args(["-m", "unittest", "-q", "anycode_original_tests"])
        .current_dir(dir)
        // The harness's own runs must not leave __pycache__ in the repo the agent inspects.
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false);
    let _ = std::fs::remove_file(copy);
    passed
}

fn one_line(value: &Value, max: usize) -> String {
    let text = value.to_string().replace("\\n", " ");
    if text.chars().count() <= max {
        text
    } else {
        format!("{}…", text.chars().take(max).collect::<String>())
    }
}

#[test]
#[ignore = "needs a local Ollama with a tool-capable model; slow on CPU"]
fn an_agent_implements_and_verifies_a_repository_task() {
    let model = std::env::var("ANYCODE_LIVE_MODEL").unwrap_or_else(|_| "qwen2.5:3b".into());
    let provider = std::env::var("ANYCODE_LIVE_PROVIDER").unwrap_or_else(|_| "ollama".into());
    let repo = make_repo();
    assert!(
        !suite_passes(&repo),
        "precondition: the suite must fail before the task"
    );
    assert!(
        !original_suite_passes(&repo),
        "precondition: the original tests must fail before the task"
    );
    println!("repository: {}", repo.display());
    println!("model:      {provider}/{model}\n");

    let app = tauri::test::mock_builder()
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("mock app should build");
    app.manage(AppState {
        store: Mutex::new(Store::open_in_memory().unwrap()),
        workspace: Mutex::new(Some(WorkspaceState {
            fs_root: anycode_fs::WorkspaceRoot::new(&repo).unwrap(),
        })),
        terminals: Mutex::new(HashMap::new()),
        tools: ToolRegistry::standard(),
        pending_approvals: Mutex::new(HashMap::new()),
        running_tasks: Mutex::new(HashMap::new()),
        index: Mutex::new(Default::default()),
        session_id: Uuid::new_v4(),
    });
    if provider == "openai_compatible" {
        let state = app.state::<AppState>();
        state
            .store
            .lock()
            .unwrap()
            .set_setting(
                "provider.openai_compatible.base_url",
                "http://localhost:11434/v1",
            )
            .unwrap();
    }
    let handle = app.handle().clone();

    // Phase 4: a memory the user wrote, and the real index, built the way opening the
    // workspace builds it. The task must not start until the index is ready, or the run
    // would test the "not ready" path instead.
    {
        let state = app.state::<AppState>();
        let store = state.store.lock().unwrap();
        store
            .add_memory(
                anycode_store::MemoryScope::Global,
                None,
                "Keep changes minimal; do not add dependencies.",
                None,
            )
            .unwrap();
    }
    let root = anycode_fs::WorkspaceRoot::new(&repo)
        .unwrap()
        .path()
        .to_path_buf();
    let (index_tx, index_rx) = mpsc::channel::<Value>();
    handle.listen("index:status", move |event| {
        let _ = index_tx.send(serde_json::from_str(event.payload()).unwrap_or(Value::Null));
    });
    crate::index_commands::start(&handle, root);
    loop {
        let status = index_rx
            .recv_timeout(Duration::from_secs(120))
            .expect("the index never reported ready");
        println!("index:     {status}");
        match status["state"].as_str() {
            Some("ready") => break,
            Some("failed") => panic!("indexing failed: {status}"),
            _ => {}
        }
    }

    let task_uuid = Uuid::new_v4();
    let task_id = task_uuid.to_string();
    let (tx, rx) = mpsc::channel::<(&'static str, Value)>();
    for channel in CHANNELS {
        let tx = tx.clone();
        handle.listen(format!("task:{channel}:{task_id}"), move |event| {
            let payload = serde_json::from_str(event.payload()).unwrap_or(Value::Null);
            let _ = tx.send((channel, payload));
        });
    }

    let started = Instant::now();
    run_task(
        handle.clone(),
        task_id.clone(),
        provider,
        model,
        Uuid::new_v4().to_string(),
        INSTRUCTION.into(),
    )
    .expect("task should start");

    let deadline = started + Duration::from_secs(45 * 60);
    let mut approvals = 0;
    let mut context: Option<Value> = None;
    let (outcome, payload) = loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let (channel, payload) = rx
            .recv_timeout(remaining)
            .expect("the task never finished within the deadline");
        let t = started.elapsed().as_secs();
        match channel {
            "state" => println!("[{t:>4}s] state      {}", payload["state"]),
            "plan" => {
                println!("[{t:>4}s] plan");
                for (i, step) in payload["steps"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .enumerate()
                {
                    println!("         {}. {}", i + 1, step.as_str().unwrap_or_default());
                }
            }
            "context" => {
                println!(
                    "[{t:>4}s] context    ~{} of ~{} repository tokens (estimated)",
                    payload["estTokens"], payload["repoEstTokens"]
                );
                for item in payload["items"].as_array().into_iter().flatten() {
                    println!(
                        "         {}:{}-{} {}",
                        item["path"].as_str().unwrap_or_default(),
                        item["startLine"],
                        item["endLine"],
                        one_line(&item["reasons"], 100)
                    );
                }
                context = Some(payload);
            }
            "replan" => println!("[{t:>4}s] replan     {}", one_line(&payload["reason"], 160)),
            "tool_call" => println!(
                "[{t:>4}s] call       {} [{}] {}",
                payload["name"].as_str().unwrap_or_default(),
                payload["risk"].as_str().unwrap_or_default(),
                one_line(&payload["arguments"], 140)
            ),
            "approval_requested" => {
                approvals += 1;
                println!(
                    "[{t:>4}s] APPROVED   {} {} (as the user, allow once)",
                    payload["name"].as_str().unwrap_or_default(),
                    one_line(&payload["arguments"], 120)
                );
                respond_to_approval(
                    handle.clone(),
                    payload["id"].as_str().unwrap().to_string(),
                    ApprovalResponse::AllowOnce,
                )
                .expect("approval should be delivered");
            }
            "tool_result" => println!("[{t:>4}s] result     {}", one_line(&payload["result"], 160)),
            terminal @ ("done" | "error" | "cancelled") => break (terminal, payload),
            _ => {}
        }
    };

    println!("\n[{:>4}s] {outcome}", started.elapsed().as_secs());
    println!("{}", serde_json::to_string_pretty(&payload).unwrap());

    assert_eq!(outcome, "done", "the task did not finish normally");

    // Checked before the verdict: a task that fails must still leave a complete record.
    let state = app.state::<AppState>();
    let events = state.store.lock().unwrap().task_events(task_uuid).unwrap();
    let kinds: Vec<&str> = events.iter().map(|e| e.kind.as_str()).collect();
    for required in [
        "task.created",
        "task.context",
        "task.memories",
        "task.plan",
        "task.tool.call",
        "task.approval",
        "task.tool.result",
        "task.finished",
    ] {
        assert!(
            kinds.contains(&required),
            "audit log is missing {required}: {kinds:?}"
        );
    }

    let context = context.expect("the task emitted no context package");
    let paths: Vec<&str> = context["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|i| i["path"].as_str())
        .collect();
    assert!(
        paths.contains(&"calc.py"),
        "context lacks calc.py: {paths:?}"
    );

    assert_eq!(
        payload["verdict"]["kind"], "passed",
        "the runtime did not judge the work verified"
    );
    let changed: Vec<&str> = payload["evidence"]["filesChanged"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        changed.contains(&"calc.py"),
        "calc.py should have changed: {changed:?}"
    );

    // Don't take the runtime's word for it either — and judge by the tests as they were
    // committed, not by whatever the agent may have edited them into.
    assert!(
        original_suite_passes(&repo),
        "the original tests fail against the agent's calc.py"
    );

    println!(
        "\naudit log: {} events ({kinds:?}); {approvals} approval(s) answered; original \
         tests pass independently",
        events.len()
    );

    let _ = std::fs::remove_dir_all(&repo);
}
