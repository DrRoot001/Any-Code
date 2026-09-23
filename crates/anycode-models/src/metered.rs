//! Usage metering at the adapter boundary (docs/ARCHITECTURE.md invariant #9: "the event
//! is emitted at the adapter, not the call site").
//!
//! Every provider the application builds is wrapped in [`Metered`], so each streamed
//! request is reported exactly once — whoever called it, and however it ended — without
//! any call site having to remember. Before this, usage was recorded by hand at each call
//! site, and a request cancelled mid-stream was not recorded at all, although it consumed
//! tokens.

use crate::provider::{ModelProvider, ModelStream};
use crate::types::{ModelDefinition, ModelRequest, ProviderError, ProviderManifest, StreamEvent};
use async_trait::async_trait;
use futures_core::Stream;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageOutcome {
    Success,
    Error,
    /// The caller stopped reading before the provider finished. Tokens were probably
    /// consumed; the provider never said how many, so none are reported.
    Cancelled,
}

/// One model request, as it actually went. Token counts are what the provider reported —
/// `None` when it reported nothing, never an estimate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageReport {
    pub provider: String,
    pub model: String,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub outcome: UsageOutcome,
}

pub type UsageSink = Arc<dyn Fn(UsageReport) + Send + Sync>;

/// Wraps any provider so every `stream` call is metered. Model listing is not metered:
/// it is not a model request.
pub struct Metered {
    provider_id: String,
    inner: Box<dyn ModelProvider>,
    sink: UsageSink,
}

impl Metered {
    pub fn new(
        provider_id: impl Into<String>,
        inner: Box<dyn ModelProvider>,
        sink: UsageSink,
    ) -> Self {
        Self {
            provider_id: provider_id.into(),
            inner,
            sink,
        }
    }
}

#[async_trait]
impl ModelProvider for Metered {
    fn manifest(&self) -> ProviderManifest {
        self.inner.manifest()
    }

    async fn models(&self) -> Result<Vec<ModelDefinition>, ProviderError> {
        self.inner.models().await
    }

    async fn stream(&self, request: ModelRequest) -> Result<ModelStream, ProviderError> {
        let mut report = Report {
            sink: self.sink.clone(),
            provider: self.provider_id.clone(),
            model: request.model.clone(),
            done: false,
        };
        match self.inner.stream(request).await {
            Ok(inner) => Ok(Box::pin(MeteredStream { inner, report })),
            Err(err) => {
                report.send(None, None, UsageOutcome::Error);
                Err(err)
            }
        }
    }
}

/// Sends one report per request. If it is dropped without having sent — the stream was
/// abandoned — it reports a cancellation, so no request goes unrecorded.
struct Report {
    sink: UsageSink,
    provider: String,
    model: String,
    done: bool,
}

impl Report {
    fn send(&mut self, input: Option<u32>, output: Option<u32>, outcome: UsageOutcome) {
        if self.done {
            return;
        }
        self.done = true;
        (self.sink)(UsageReport {
            provider: self.provider.clone(),
            model: self.model.clone(),
            input_tokens: input,
            output_tokens: output,
            outcome,
        });
    }
}

impl Drop for Report {
    fn drop(&mut self) {
        self.send(None, None, UsageOutcome::Cancelled);
    }
}

struct MeteredStream {
    inner: ModelStream,
    report: Report,
}

impl Stream for MeteredStream {
    type Item = Result<StreamEvent, ProviderError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let item = this.inner.as_mut().poll_next(cx);
        match &item {
            Poll::Ready(Some(Ok(StreamEvent::Done { usage }))) => {
                this.report.send(
                    usage.input_tokens,
                    usage.output_tokens,
                    UsageOutcome::Success,
                );
            }
            Poll::Ready(Some(Err(_))) => this.report.send(None, None, UsageOutcome::Error),
            // Ended cleanly without the provider reporting usage: it succeeded, and the
            // token counts are genuinely unknown.
            Poll::Ready(None) => this.report.send(None, None, UsageOutcome::Success),
            _ => {}
        }
        item
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Message, ProviderAuthMode, RequestMetadata, Usage};
    use futures_util::StreamExt;
    use std::sync::Mutex;

    /// Test double: streams a fixed script of events. Verification scaffolding only.
    struct Scripted {
        open_fails: bool,
        events: Vec<Result<StreamEvent, &'static str>>,
    }

    #[async_trait]
    impl ModelProvider for Scripted {
        fn manifest(&self) -> ProviderManifest {
            ProviderManifest {
                id: "scripted",
                name: "Scripted",
                auth_modes: &[ProviderAuthMode::Local],
                supports_streaming: true,
                supports_tools: false,
                supports_vision: false,
            }
        }
        async fn models(&self) -> Result<Vec<ModelDefinition>, ProviderError> {
            Ok(vec![])
        }
        async fn stream(&self, _request: ModelRequest) -> Result<ModelStream, ProviderError> {
            if self.open_fails {
                return Err(ProviderError::Api("refused".into()));
            }
            let items: Vec<_> = self
                .events
                .iter()
                .cloned()
                .map(|e| e.map_err(|m| ProviderError::Api(m.into())))
                .collect();
            Ok(Box::pin(futures_util::stream::iter(items)))
        }
    }

    fn metered(
        open_fails: bool,
        events: Vec<Result<StreamEvent, &'static str>>,
    ) -> (Metered, Arc<Mutex<Vec<UsageReport>>>) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink_seen = seen.clone();
        let sink: UsageSink = Arc::new(move |r| sink_seen.lock().unwrap().push(r));
        (
            Metered::new("scripted", Box::new(Scripted { open_fails, events }), sink),
            seen,
        )
    }

    fn request() -> ModelRequest {
        ModelRequest {
            model: "m1".into(),
            messages: vec![Message::user("hi")],
            temperature: None,
            tools: None,
            metadata: RequestMetadata::default(),
        }
    }

    fn text(t: &str) -> Result<StreamEvent, &'static str> {
        Ok(StreamEvent::TextDelta { text: t.into() })
    }

    fn done(input: u32, output: u32) -> Result<StreamEvent, &'static str> {
        Ok(StreamEvent::Done {
            usage: Usage {
                input_tokens: Some(input),
                output_tokens: Some(output),
            },
        })
    }

    #[tokio::test]
    async fn a_completed_request_is_reported_once_with_real_counts() {
        let (provider, seen) = metered(false, vec![text("a"), done(12, 4)]);
        let events: Vec<_> = provider.stream(request()).await.unwrap().collect().await;
        assert_eq!(events.len(), 2, "the wrapper must not alter the stream");
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].outcome, UsageOutcome::Success);
        assert_eq!(
            (seen[0].input_tokens, seen[0].output_tokens),
            (Some(12), Some(4))
        );
        assert_eq!(
            (seen[0].provider.as_str(), seen[0].model.as_str()),
            ("scripted", "m1")
        );
    }

    #[tokio::test]
    async fn failures_are_reported_whether_at_open_or_mid_stream() {
        let (provider, seen) = metered(true, vec![]);
        assert!(provider.stream(request()).await.is_err());
        let (provider, seen_mid) = metered(false, vec![text("a"), Err("boom")]);
        let _: Vec<_> = provider.stream(request()).await.unwrap().collect().await;
        for seen in [seen, seen_mid] {
            let seen = seen.lock().unwrap();
            assert_eq!(seen.len(), 1);
            assert_eq!(seen[0].outcome, UsageOutcome::Error);
        }
    }

    #[tokio::test]
    async fn an_abandoned_stream_is_reported_as_cancelled() {
        let (provider, seen) = metered(false, vec![text("a"), text("b"), done(1, 1)]);
        let mut stream = provider.stream(request()).await.unwrap();
        let _first = stream.next().await;
        drop(stream);
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].outcome, UsageOutcome::Cancelled);
        assert_eq!(seen[0].input_tokens, None, "counts are never estimated");
    }

    #[tokio::test]
    async fn a_clean_end_without_usage_is_success_with_unknown_counts() {
        let (provider, seen) = metered(false, vec![text("a")]);
        let _: Vec<_> = provider.stream(request()).await.unwrap().collect().await;
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].outcome, UsageOutcome::Success);
        assert_eq!(seen[0].output_tokens, None);
    }
}
