use criterion::{criterion_group, criterion_main, Criterion};
use hangul::Hangul;
use serde::Deserialize;

#[derive(Deserialize)]
struct Scenario {
  label: String,
  text: String,
}

// Shared with the JavaScript benchmarks so both harnesses measure identical inputs.
fn get_scenarios() -> Vec<Scenario> {
  let raw = include_str!("../fixtures/scenarios.json");
  serde_json::from_str(raw).expect("fixtures/scenarios.json must be a valid scenario list")
}

fn bench_scenarios(c: &mut Criterion) {
  let scenarios = get_scenarios();

  for scenario in &scenarios {
    let text = &scenario.text;
    let mut group = c.benchmark_group(&scenario.label);

    group.sample_size(10000);

    group.bench_function("rusty-hangul", |b| {
      b.iter(|| {
        let hangul = Hangul::new(text);
        let _ = hangul.disassemble();
        let _ = hangul.get_choseong();
      })
    });

    group.finish();
  }
}

criterion_group!(benches, bench_scenarios);
criterion_main!(benches);
