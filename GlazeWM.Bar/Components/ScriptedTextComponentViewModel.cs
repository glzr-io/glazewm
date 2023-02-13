using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Concurrency;
using System.Reactive.Linq;
using System.Text;
using System.Text.Json;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using CliWrap;
using GlazeWM.Domain.UserConfigs;

namespace GlazeWM.Bar.Components;

public class ScriptedTextComponentViewModel : ComponentViewModel
{
  private readonly ScriptedTextConfig _baseConfig;
  private static readonly Regex Regex = new(@"\{[^}]+\}", RegexOptions.Compiled);

  public string FormattedText { get; set; }

  public ScriptedTextComponentViewModel(BarViewModel parentViewModel, ScriptedTextConfig baseConfig) : base(parentViewModel, baseConfig)
  {
    _baseConfig = baseConfig;
    var updateInterval = TimeSpan.FromMilliseconds(_baseConfig.IntervalMs);
    Observable.Interval(updateInterval)
      .Subscribe( _ => RunScript());
  }

  private async Task RunScript()
  {
    var outSb = new StringBuilder();
    var res = await Cli.Wrap("dotnet")
      .WithArguments($"fsi {_baseConfig.ScriptPath} {_baseConfig.Args}")
      .WithStandardOutputPipe(PipeTarget.ToStringBuilder(outSb))
      .ExecuteAsync();
    var output = outSb.ToString();
    var json = Regex.Matches(output).Last(); // extract json block
    var result = JsonSerializer.Deserialize<Dictionary<string, object>>(json.Value);
    FormattedText = result.Aggregate(_baseConfig.Format, (state, item) => state.Replace($"{{{item.Key}}}", item.Value.ToString()));
    OnPropertyChanged(nameof(FormattedText));
  }
}
