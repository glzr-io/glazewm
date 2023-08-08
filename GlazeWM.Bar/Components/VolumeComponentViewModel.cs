using System;
using System.Collections.Generic;
using System.Globalization;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components
{
  public class VolumeComponentViewModel : ComponentViewModel
  {
    private readonly VolumeComponentConfig _config;

    private readonly SystemVolumeInformation _sysVolume =
      ServiceLocator.GetRequiredService<SystemVolumeInformation>();

    private LabelViewModel _label;
    public LabelViewModel Label
    {
      get => _label;
      protected set => SetField(ref _label, value);
    }

    public VolumeComponentViewModel(
      BarViewModel parentViewModel,
      VolumeComponentConfig config) : base(parentViewModel, config)
    {
      _config = config;

      var initVolume = _sysVolume.GetVolumeInformation();
      Label = CreateLabel(initVolume);

      _sysVolume.VolumeChanged += (_, volumeInfo) =>
      {
        Label = CreateLabel(volumeInfo);
      };
    }

    private string GetVolumeLabel(VolumeInformation volumeInfo)
    {
      if (volumeInfo.Muted)
        return _config.LabelMute;

      return volumeInfo.Volume switch
      {
        > 0 and < 33 => _config.LabelLow,
        >= 33 and < 66 => _config.LabelMedium,
        _ => _config.LabelHigh
      };
    }

    public LabelViewModel CreateLabel(VolumeInformation volumeInfo)
    {
      return XamlHelper.ParseLabel(
        GetVolumeLabel(volumeInfo),
        CreateVariableDict(volumeInfo),
        this
      );
    }

    public static Dictionary<string, Func<string>> CreateVariableDict(
      VolumeInformation volumeInfo)
    {
      return new()
      {
        { "volume_level", () => volumeInfo.Volume.ToString("0", CultureInfo.InvariantCulture) },
      };
    }
  }
}
