using System;
using System.Diagnostics;
using System.Reactive.Linq;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Events;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Bar.Components
{
  public class VolumeComponentViewModel : ComponentViewModel
  {
    private string _formattedText = "ABC";
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

      sysVolume.VolumeChangedEvent += (_, args) =>
      {
        FormattedText = ">" + (args.Volume * 100).ToString("0");
        Debug.WriteLine(FormattedText);
      };

      // var sysControls = SystemMediaTransportControls.GetForCurrentView();

      // _bus.Events.OfType<WindowFocusedEvent>()
      //   .Subscribe((@event) => UpdateTitle(@event.WindowHandle));

      // _bus.Events.OfType<WindowTitleChangedEvent>()
      //   .Subscribe((@event) => UpdateTitle(@event.WindowHandle));
    }
  }
}
