## Enable Profiling

```
echo 1 | sudo tee /proc/sys/kernel/perf_event_paranoid
```

## Run Profile Test

```
python record -g python tests/models/test_pso.py
```

## Disable Profiling

```
echo 4| sudo tee /proc/sys/kernel/perf_event_paranoid
```

## See Report

```
perf report
```
