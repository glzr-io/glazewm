using System;
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
    public string Text => FormatLabel();
    private string FormatLabel()
    {
      if (!NetworkService.PingTest())
        return _config.LabelNoInternet;
      return getNetworkIcon();
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
      var updateSubscription = Observable
                               .Interval(TimeSpan.FromSeconds(10))
                               .Subscribe(_ => OnPropertyChanged(nameof(Text)));

      RegisterDisposables(updateSubscription);

      NetworkChange.NetworkAddressChanged += new NetworkAddressChangedEventHandler((s, e) => OnPropertyChanged(nameof(Text)));
    }
  }
}
