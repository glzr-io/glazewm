using System.Diagnostics;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components
{
  public class VolumeComponentViewModel : ComponentViewModel
  {
    private VolumeComponentConfig _config => _componentConfig as VolumeComponentConfig;
    private readonly SystemVolumeInformation _sysVolume = ServiceLocator.GetRequiredService<SystemVolumeInformation>();
    private string _formattedText;
    public string FormattedText
    {
      get => _formattedText;
      set
      {
        _formattedText = value;
        OnPropertyChanged(nameof(FormattedText));
      }
    }
    public VolumeComponentViewModel(
      BarViewModel parentViewModel,
      VolumeComponentConfig config) : base(parentViewModel, config)
    {
      var initVolume = _sysVolume.GetVolumeInformation();
      FormattedText = GetVolumeIcon(initVolume) + initVolume.Volume.ToString("00");

      _sysVolume.VolumeChanged += (_, volumeInfo) =>
      {
        FormattedText = GetVolumeIcon(volumeInfo) + volumeInfo.Volume.ToString("00");
        Debug.WriteLine(FormattedText);
      };
    }

    private string GetVolumeIcon(VolumeInformation vol)
    {
      if (vol.Muted)
        return _config.LabelVolumeMute;

      if (vol.Volume < 10)
        return _config.LabelVolumeLow;
      else if (vol.Volume < 50)
        return _config.LabelVolumeMed;
      else if (vol.Volume < 100)
        return _config.LabelVolumeHigh;

      return _config.LabelVolumeLow;
    }
  }
}
