use criterion::{black_box, criterion_group, criterion_main, Criterion};
use api_warden::processor::{*};
use rand::prelude::*;

mod perf;


fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let mut sample_input = vec![];

    for i in 0..100 {
        let sample = format!(
            r#"{{
                "source":"{app}", 
                "uri":"http://test.app.com", 
                "req_hdr":{{"a":"b", "another-header":"and-its-value"}},
                "method":"get",
                "req":{{}},
                "resp_hdr":{{"c":"d", "header-2":"thy value"}},
                "resp":{{"page":{counter}, "per_page":10, "x":[], "data":[{{"id":{counter}, "name":"sdffsdfsdh{counter}"}},{{"id":{counter2}, "name":"35613444-hhh"}}]}},
                "ts":12999884423423
            }}"#,
            app = format!("app-{}", rng.gen_range(1..7)),
            counter = i,
            counter2 = i + 3
        );

        sample_input.push(sample);
    }

    let mut p = Processor::new();

    c.bench_function(
        "process_transaction", 
        |b| b.iter(
            || for sample in sample_input.iter() {
                let _ = p.process_transaction(&sample);
            }) );
}

criterion_group!{
    name = benches;
    config = Criterion::default().with_profiler(perf::FlamegraphProfiler::new(100));
    targets = criterion_benchmark
}

criterion_main!(benches);