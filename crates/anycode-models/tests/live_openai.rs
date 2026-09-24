//! Live contract check against the real OpenAI API.
//!
//! Ignored by default: it needs a real key and makes a real (small) billable request, so
//! it must never run in CI. It exists because the riskiest part of the agent runtime is
//! the one part unit tests can't reach — whether a *real* streaming response actually
//! produces a `ToolCall` through our SSE fragment reassembly.
//!
//! Run it with:
//!
//! ```sh
//! OPENAI_API_KEY=sk-... cargo test -p anycode-models --test live_openai -- --ignored --nocapture
//! ```

use anycode_models::{
    Message, ModelProvider, ModelRequest, OpenAiProvider, RequestMetadata, StreamEvent,
    ToolDefinition,
};
use futures_util::StreamExt;
use serde_json::json;

fn api_key() -> String {
    std::env::var("OPENAI_API_KEY").expect("set OPENAI_API_KEY to run this test")
}

fn model() -> String {
    // Overridable so this doesn't rot when the cheap model of the day changes.
    std::env::var("ANYCODE_TEST_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string())
}

fn read_file_tool() -> ToolDefinition {
    ToolDefinition {
        name: "filesystem.read.workspace".to_string(),
        description: "Read a text file's full contents from the open workspace.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": { "path": { "type": "string" } },
            "required": ["path"],
        }),
    }
}

#[tokio::test]
#[ignore = "makes a real billable API request; needs OPENAI_API_KEY"]
async fn a_real_response_produces_a_tool_call() {
    let provider = OpenAiProvider::new(api_key());
    let request = ModelRequest {
        model: model(),
        max_output_tokens: None,
        messages: vec![Message::user(
            "Read the file README.md using the provided tool. Call the tool; do not answer \
             from memory.",
        )],
        temperature: Some(0.0),
        tools: Some(vec![read_file_tool()]),
        metadata: RequestMetadata::default(),
    };

    let mut stream = provider.stream(request).await.expect("stream should open");
    let mut tool_calls = Vec::new();

    while let Some(event) = stream.next().await {
        match event.expect("stream should not error") {
            StreamEvent::ToolCall {
                name, arguments, ..
            } => tool_calls.push((name, arguments)),
            StreamEvent::TextDelta { .. } | StreamEvent::Done { .. } => {}
        }
    }

    let (name, arguments) = tool_calls
        .first()
        .expect("the model should have requested a tool call");
    assert_eq!(name, "filesystem.read.workspace");
    assert_eq!(
        arguments["path"], "README.md",
        "arguments should be parsed JSON, not a raw string: {arguments:?}"
    );
}

#[tokio::test]
#[ignore = "makes a real billable API request; needs OPENAI_API_KEY"]
async fn a_real_response_reports_usage() {
    let provider = OpenAiProvider::new(api_key());
    let request = ModelRequest {
        model: model(),
        max_output_tokens: None,
        messages: vec![Message::user("Reply with exactly: ok")],
        temperature: Some(0.0),
        tools: None,
        metadata: RequestMetadata::default(),
    };

    let mut stream = provider.stream(request).await.expect("stream should open");
    let mut usage_seen = None;

    while let Some(event) = stream.next().await {
        if let StreamEvent::Done { usage } = event.expect("stream should not error") {
            usage_seen = Some(usage);
        }
    }

    let usage = usage_seen.expect("the final chunk should carry usage");
    assert!(
        usage.input_tokens.unwrap_or(0) > 0,
        "input tokens should be reported, got {usage:?}"
    );
}
