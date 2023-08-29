// use std::borrow::Cow;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::fmt::Debug;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Benchmark<'a> {
    name: &'a str,
    config: BenchmarkConfig,
    timings: Vec<(&'a str, BenchmarkResult<'a>)>,
}

impl<'a> Benchmark<'a> {
    pub fn new(name: &'a str) -> Self {
        Benchmark {
            name,
            config: BenchmarkConfig::default(),
            timings: Vec::new(),
        }
    }

    pub fn with_config(name: &'a str, config: BenchmarkConfig) -> Self {
        Benchmark {
            name,
            config,
            timings: Vec::new(),
        }
    }

    pub fn from_env(name: &'a str) -> Self {
        Benchmark {
            name,
            config: BenchmarkConfig::from_env(),
            timings: Vec::new(),
        }
    }

    pub fn benchmark<F: Fn(&mut BenchmarkRun)>(&mut self, name: &'a str, func: F) {
        let mut run = BenchmarkRun::new(name, "");
        func(&mut run);
        self.timings.push((name, BenchmarkResult { name, run }));
    }

    pub fn benchmark_with<F: Fn(&mut BenchmarkRun, &P) -> T, T, P: Debug>(
        &mut self,
        name: &'a str,
        params: &[(&'a str, P)],
        func: F,
    ) {
        for p in params
            .iter()
            .take(if self.config.quick { 1 } else { usize::MAX })
        {
            let mut run = BenchmarkRun::new(name, p.0);
            func(&mut run, &p.1);
            self.timings.push((name, BenchmarkResult { name, run }));
        }
    }

    pub fn output(&self) {
        let output = json!(self.timings);
        println!("{}", json!({ "name": self.name, "timings": output }));
    }
}

#[derive(Debug, Default)]
pub struct BenchmarkConfig {
    pub quick: bool,
}

impl BenchmarkConfig {
    pub fn from_env() -> Self {
        let quick = env::var("BENCH_QUICK").unwrap_or("false".to_string());
        BenchmarkConfig {
            quick: quick == "true" || quick == "1",
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BenchmarkResult<'a> {
    pub name: &'a str,
    pub run: BenchmarkRun<'a>,
}

impl<'a> BenchmarkResult<'a> {
    pub fn new(name: &'a str, run: BenchmarkRun<'a>) -> Self {
        BenchmarkResult { name, run }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BenchmarkRun<'a> {
    pub name: &'a str,
    pub param: &'a str,
    pub time: Duration,
    pub metrics: HashMap<&'a str, usize>,
}

impl<'a> BenchmarkRun<'a> {
    fn new(name: &'a str, param: &'a str) -> Self {
        BenchmarkRun {
            name,
            param,
            time: Duration::new(0, 0),
            metrics: HashMap::new(),
        }
    }

    pub fn run<F, R>(&mut self, func: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start_time = Instant::now();
        let out = func();
        let elapsed_time = start_time.elapsed();
        self.time = elapsed_time;
        out
    }

    pub fn log(&mut self, metric: &'a str, value: usize) {
        self.metrics.insert(metric, value);
    }
}
