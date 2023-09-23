# Evaluation for sending

## Communication additional payload for ID

164 B

Steps:

1. Run `eval-gen-sender-pk-sk` and follow the output to set envs
2. Run `eval-eems-for-send`
3. Run `eval-msg-id-size` and get the result from the output

## Time of ID generation

Message body size: $2^3$ - $2^{13}$ (B)

Local gRPC is set up

Steps:

1. Run `eval-gen-sender-pk-sk` and follow the output to set envs
2. Run `eval-eems-for-send`
3. Run `cargo bench --bench gen_id` and get the result from the output

Raw output:

```text
Benchmarking gen_id/8: Collecting 100 samples
gen_id/8                time:   [931.52 µs 936.45 µs 941.97 µs]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

Benchmarking gen_id/16: Warming up for 3.0000
Benchmarking gen_id/16: Collecting 100 samples
gen_id/16               time:   [939.70 µs 943.90 µs 948.07 µs]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe

Benchmarking gen_id/32: Warming up for 3.0000
Benchmarking gen_id/32: Collecting 100 samples
gen_id/32               time:   [934.64 µs 939.16 µs 943.98 µs]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

Benchmarking gen_id/64: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.0s, enable flat sampling, or reduce sample count to 70.
Benchmarking gen_id/64: Collecting 100 samples
gen_id/64               time:   [967.64 µs 974.60 µs 983.11 µs]
Found 13 outliers among 100 measurements (13.00%)
  10 (10.00%) high mild
  3 (3.00%) high severe

Benchmarking gen_id/128: Warming up for 3.0000
Benchmarking gen_id/128: Collecting 100 sample
gen_id/128              time:   [984.70 µs 991.86 µs 999.11 µs]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe

Benchmarking gen_id/256: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.1s, enable flat sampling, or reduce sample count to 60.
Benchmarking gen_id/256: Collecting 100 sample
gen_id/256              time:   [992.70 µs 998.75 µs 1.0049 ms]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  2 (2.00%) high severe

Benchmarking gen_id/512: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.1s, enable flat sampling, or reduce sample count to 70.
Benchmarking gen_id/512: Collecting 100 sample
gen_id/512              time:   [998.18 µs 1.0045 ms 1.0111 ms]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  6 (6.00%) high mild

Benchmarking gen_id/1024: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.0s, enable flat sampling, or reduce sample count to 70.
Benchmarking gen_id/1024: Collecting 100 sampl
gen_id/1024             time:   [996.32 µs 1.0033 ms 1.0101 ms]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

Benchmarking gen_id/2048: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.1s, enable flat sampling, or reduce sample count to 60.
Benchmarking gen_id/2048: Collecting 100 sampl
gen_id/2048             time:   [1.0043 ms 1.0091 ms 1.0145 ms]
Found 12 outliers among 100 measurements (12.00%)
  8 (8.00%) high mild
  4 (4.00%) high severe

Benchmarking gen_id/4096: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.1s, enable flat sampling, or reduce sample count to 60.
Benchmarking gen_id/4096: Collecting 100 sampl
gen_id/4096             time:   [1.0223 ms 1.0323 ms 1.0428 ms]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
```
