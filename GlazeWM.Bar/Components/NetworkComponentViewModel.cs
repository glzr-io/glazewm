using System;
using System.Collections.Generic;
using System.Net.NetworkInformation;
using System.Reactive.Linq;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.WindowsApi;
using static Vanara.PInvoke.IpHlpApi;

namespace GlazeWM.Bar.Components
{
  public class NetworkComponentViewModel : ComponentViewModel
  {
    private NetworkComponentConfig _config => _componentConfig as NetworkComponentConfig;

    /// <summary>
    /// Format the current power status with the user's formatting config.
    /// </summary>
    private LabelViewModel _label;

    public LabelViewModel Label
    {
      get => _label;
      protected set => SetField(ref _label, value);
    }

    private string getNetworkIcon()
    {
      var primaryAdapter = NetworkService.GetPrimaryAdapter();
      switch (primaryAdapter.IfType)
      {
        case IFTYPE.IF_TYPE_ETHERNET_CSMACD:
        case IFTYPE.IF_TYPE_ETHERNET_3MBIT:
          // Primary adapter is using ethernet.
          return _config.LabelEthernet;
        case IFTYPE.IF_TYPE_IEEE80211:
          // Primary adapter is using wifi.
          var rssi = NetworkService.GetWlanRSSI();
          return assignWifiIcon(rssi);
        default:
          return _config.LabelNoInternet;
      }
    }

    private string assignWifiIcon(int signalQuality)
    {
      // Round to nearest multiple and assign icon.  
      return ((signalQuality % 25) > (25 / 2) ? (signalQuality / 25) + 1 : signalQuality / 25) switch
      {
        0 => _config.LabelWifiStrength0,
        1 => _config.LabelWifiStrength25,
        2 => _config.LabelWifiStrength50,
        3 => _config.LabelWifiStrength75,
        4 => _config.LabelWifiStrength100,
        _ => _config.LabelNoInternet,
      };
    }

    public NetworkComponentViewModel(
      BarViewModel parentViewModel,
      NetworkComponentConfig config) : base(parentViewModel, config)
    {
      Observable
        .Interval(TimeSpan.FromSeconds(10))
        .TakeUntil(_parentViewModel.WindowClosing)
        .Subscribe(_ => Label = CreateLabel());

      NetworkChange.NetworkAddressChanged +=
        (s, e) => Label = CreateLabel();
    }

    public LabelViewModel CreateLabel()
    {
      string icon;
      if (!NetworkService.PingTest())
        icon = _config.LabelNoInternet;
      else
        icon = getNetworkIcon();

      return XamlHelper.ParseLabel(
        icon,
        null,
        this
      );
    }
  }
}
