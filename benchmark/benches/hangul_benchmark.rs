use std::hint::black_box;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use hangul::{assemble, Hangul};
use serde::Deserialize;

#[derive(Deserialize)]
struct Scenario {
  id: String,
  label: String,
  text: String,
}

#[derive(Deserialize)]
struct Workload {
  id: String,
  label: String,
  scenarios: Vec<String>,
  #[serde(default)]
  args: Vec<String>,
}

fn parse<T: for<'de> Deserialize<'de>>(raw: &str, name: &str) -> T {
  serde_json::from_str(raw)
    .unwrap_or_else(|error| panic!("{name} must be a valid fixture: {error}"))
}

fn load_scenarios() -> Vec<Scenario> {
  parse(
    include_str!("../fixtures/scenarios.json"),
    "fixtures/scenarios.json",
  )
}

fn load_workloads() -> Vec<Workload> {
  parse(
    include_str!("../fixtures/workloads.json"),
    "fixtures/workloads.json",
  )
}

fn run_workload(workload: &Workload, text: &str) {
  let arg = workload
    .args
    .first()
    .map(String::as_str)
    .unwrap_or_default();
  let hangul = Hangul::new(black_box(text));

  match workload.id.as_str() {
    "disassemble-choseong" => {
      black_box(hangul.disassemble());
      black_box(hangul.get_choseong());
    }
    "disassemble-groups" => {
      black_box(hangul.disassemble_to_groups());
    }
    "has-batchim" => {
      black_box(hangul.has_batchim());
    }
    "josa" => {
      black_box(hangul.josa(arg).ok());
    }
    "assemble" => {
      black_box(assemble(black_box(text)));
    }
    "contains-choseong" => {
      black_box(hangul.contains_choseong(arg));
    }
    "find-choseong" => {
      black_box(
        hangul
          .find_choseong(arg)
          .map(|matched| (matched.start, matched.end)),
      );
    }
    other => panic!("fixtures/workloads.json contains an unknown workload id: {other}"),
  }
}

fn bench_scenarios(c: &mut Criterion) {
  let scenarios = load_scenarios();
  let workloads = load_workloads();

  for workload in &workloads {
    for scenario_id in &workload.scenarios {
      let scenario = scenarios
        .iter()
        .find(|candidate| &candidate.id == scenario_id)
        .unwrap_or_else(|| {
          panic!("fixtures/workloads.json references an unknown scenario id: {scenario_id}")
        });

      // The label matches the JavaScript benchmark label so both harnesses can be
      // merged into one report.
      let mut group = c.benchmark_group(format!("{} · {}", workload.label, scenario.label));

      // 10k samples per group made the suite dominated by outliers; a few hundred
      // samples over a fixed measurement window is enough for a stable mean.
      group
        .sample_size(200)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3));

      group.bench_function("rusty-hangul", |b| {
        b.iter(|| run_workload(workload, &scenario.text))
      });

      group.finish();

      println!();
    }
  }
}

criterion_group!(benches, bench_scenarios);
criterion_main!(benches);
