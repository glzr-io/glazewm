using System.Collections.Generic;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components
{
  public class VolumeComponentViewModel : ComponentViewModel
  {
    private VolumeComponentConfig _config => _componentConfig as VolumeComponentConfig;
    private readonly SystemVolumeInformation _sysVolume = ServiceLocator.GetRequiredService<SystemVolumeInformation>();
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
      var initVolume = _sysVolume.GetVolumeInformation();
      Label = CreateLabel(initVolume);
      _sysVolume.VolumeChanged += (_, volumeInfo) =>
      {
        Label = CreateLabel(volumeInfo);
      };
    }

    private string GetVolumeIcon(VolumeInformation vol)
    {
      if (vol.Muted)
        return _config.IconVolumeMute;

      if (vol.Volume < 10)
        return _config.IconVolumeLow;
      else if (vol.Volume < 50)
        return _config.IconVolumeMed;
      else if (vol.Volume <= 100)
        return _config.IconVolumeHigh;

      return _config.IconVolumeLow;
    }

    public LabelViewModel CreateLabel(VolumeInformation vol)
    {
      var volumeLevel = vol.Volume.ToString("00");
      var volumeIcon = GetVolumeIcon(vol);
      return XamlHelper.ParseLabel(
        _config.Label,
        CreateVariableDict(volumeLevel, volumeIcon),
        this
      );
    }

    public static Dictionary<string, string> CreateVariableDict(string volumeLevel, string volumeIcon)
    {
      return new()
      {
        { "volume_level", volumeLevel },
        { "volume_icon", volumeIcon }
      };
    }
  }
}
