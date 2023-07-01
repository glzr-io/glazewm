class a {
  private readonly _cpuCounter: IPerformanceCounter<double> =
    PerformanceCounterFactory.Default.CreateCounter(
      "Processor Information",
      "% Processor Utility",
      "_Total"
    );
}

Observable.Timer(
  TimeSpan.FromSeconds(0),
  TimeSpan.FromMilliseconds(_config.RefreshIntervalMs)
)
  .TakeUntil(_parentViewModel.WindowClosing)
  .Subscribe((_) => (Label = CreateLabel()));

const x = {
  cpu_usage: () =>
    _cpuStatsService.GetCpuUsage().ToString(CultureInfo.InvariantCulture),
};

const x = [
  "percent_usage",
  () =>
    _gpuStatsService
      .GetAverageLoadPercent(GpuPerformanceCategoryFlags.Graphics)
      .ToString(CultureInfo.InvariantCulture),
];

var res = await client.GetStringAsync(
  "https://api.open-meteo.com/v1/forecast?latitude=" +
    lat +
    "&longitude=" +
    lng +
    "&temperature_unit=celsius&current_weather=true&daily=sunset,sunrise&timezone=auto"
);
