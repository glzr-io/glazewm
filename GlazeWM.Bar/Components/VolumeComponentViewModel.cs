using System.Diagnostics;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components
{
  public class VolumeComponentViewModel : ComponentViewModel
  {
    private VolumeComponentConfig _config => _componentConfig as VolumeComponentConfig;
    private string _formattedText = "NEED INIT VALUE";
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
      var sysVolume = new SystemVolumeInformation();

      sysVolume.VolumeChangedEvent += (_, volumeInfo) =>
      {
        var volume = (int)(volumeInfo.Volume * 100);
        FormattedText = GetVolumeIcon(volume, volumeInfo.Muted) + volume.ToString("00");
        Debug.WriteLine(FormattedText);
      };
    }

    private string GetVolumeIcon(int volume, bool isMuted)
    {
      if (isMuted)
        return _config.LabelVolumeMute;

      if (volume < 10)
        return _config.LabelVolumeLow;
      else if (volume < 50)
        return _config.LabelVolumeMed;
      else if (volume < 100)
        return _config.LabelVolumeHigh;

      return _config.LabelVolumeLow;
    }
  }
}
