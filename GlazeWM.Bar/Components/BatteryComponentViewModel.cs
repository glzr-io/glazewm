using System;
using System.Collections.Generic;
using System.Globalization;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components
{
  public class BatteryComponentViewModel : ComponentViewModel
  {
    private readonly BatteryComponentConfig _config;

    /// <summary>
    /// Format the current power status with the user's formatting config.
    /// </summary>
    private LabelViewModel _label;
    public LabelViewModel Label
    {
      get => _label;
      protected set => SetField(ref _label, value);
    }

    public BatteryComponentViewModel(
      BarViewModel parentViewModel,
      BatteryComponentConfig config) : base(parentViewModel, config)
    {
      _config = config;

      Observable
        .Timer(TimeSpan.FromSeconds(0), TimeSpan.FromSeconds(5))
        .TakeUntil(_parentViewModel.WindowClosing)
        .Subscribe(_ => Label = CreateLabel());
    }

    public LabelViewModel CreateLabel()
    {
      WindowsApiService.GetSystemPowerStatus(out var ps);
      var batteryLevel = ps.BatteryLifePercent.ToString(CultureInfo.InvariantCulture);

      // Display the battery level as a 100% if device has no dedicated battery.
      if (ps.BatteryFlag == 128)
        return XamlHelper.ParseLabel(
          _config.LabelDraining,
          CreateVariableDict(batteryLevel),
          this
        );

      if (ps.ACLineStatus == 1)
        return XamlHelper.ParseLabel(
          _config.LabelCharging,
          CreateVariableDict(batteryLevel),
          this
        );

      if (ps.SystemStatusFlag == 1)
        return XamlHelper.ParseLabel(
          _config.LabelPowerSaver,
          CreateVariableDict(batteryLevel),
          this
        );

      return XamlHelper.ParseLabel(
        _config.LabelDraining,
        CreateVariableDict(batteryLevel),
        this
      );
    }

    public static Dictionary<string, Func<string>> CreateVariableDict(string batteryLevel)
    {
      return new()
      {
        { "battery_level", () => batteryLevel }
      };
    }
  }
}
