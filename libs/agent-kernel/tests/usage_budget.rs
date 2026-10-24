use agent_kernel::provider::responses::ResponsesStream;
use agent_kernel::provider::stream::StreamAccumulator;
use agent_kernel::*;
use serde_json::json;

fn usage(prompt_tokens: u32, completion_tokens: u32, cached_prompt_tokens: u32) -> Usage {
    Usage { prompt_tokens, completion_tokens, cached_prompt_tokens }
}

#[test]
fn cache_hits_cost_a_tenth_of_fresh_input() {
    assert_eq!(usage(1000, 50, 0).budget_units(), 1050, "no cache info keeps the old full-price accounting");
    assert_eq!(usage(1000, 50, 900).budget_units(), 100 + 90 + 50);
    assert_eq!(usage(1000, 0, 5000).budget_units(), 100, "a cached count above the prompt is clamped, never negative");
}

#[test]
fn a_long_run_re_sending_cached_history_stays_inside_the_budget() {
    // The 33-turn run that used to pause at 1,000,000: ~31k prompt tokens a turn, almost all cache hits.
    let control = RunControl::limits(None, Some(1_000_000));
    for _ in 0..33 {
        control.record(usage(31_000, 550, 30_000));
    }
    assert!(control.check().is_ok(), "{} units", control.reported_tokens());
    let raw = RunControl::limits(None, Some(1_000_000));
    for _ in 0..33 {
        raw.record(usage(31_000, 550, 0));
    }
    assert!(raw.check().is_err(), "without cache info the same run still trips the budget");
}

#[test]
fn openai_and_deepseek_cache_fields_are_read() {
    let mut openai = StreamAccumulator::default();
    openai.push(&json!({"choices": [], "usage": {"prompt_tokens": 100, "completion_tokens": 5, "prompt_tokens_details": {"cached_tokens": 64}}}), &mut |_| {}).unwrap();
    assert_eq!(openai.finish().usage, Some(usage(100, 5, 64)));

    let mut deepseek = StreamAccumulator::default();
    deepseek.push(&json!({"choices": [], "usage": {"prompt_tokens": 100, "completion_tokens": 5, "prompt_cache_hit_tokens": 80, "prompt_cache_miss_tokens": 20}}), &mut |_| {}).unwrap();
    assert_eq!(deepseek.finish().usage, Some(usage(100, 5, 80)));
}

#[test]
fn responses_cache_field_is_read() {
    let mut stream = ResponsesStream::default();
    let done = stream.push_event(&json!({"type": "response.completed", "response": {"status": "completed",
        "usage": {"input_tokens": 120, "output_tokens": 30, "input_tokens_details": {"cached_tokens": 100}}}}), &mut |_| {});
    assert_eq!(done, Ok(true));
    assert_eq!(stream.finish().usage, Some(usage(120, 30, 100)));
}
