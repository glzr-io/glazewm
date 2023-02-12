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
      if (!NetworkService.pingTest())
        return _config.IconNoInternet;
      return getNetworkIcon();
    }

    private string getNetworkIcon()
    {
      var primaryAdapter = NetworkService.getPrimaryAdapter();
      switch (primaryAdapter.IfType)
      {
        case IFTYPE.IF_TYPE_ETHERNET_CSMACD:
        case IFTYPE.IF_TYPE_ETHERNET_3MBIT:
          // Primary adapter is using ethernet.
          return _config.IconEthernet;
        case IFTYPE.IF_TYPE_IEEE80211:
          // Primary adapter is using wifi, find the primary WLAN interface.
          var rssi = NetworkService.getWlanRSSI();
          return assignWifiIcon(rssi);
        default:
          return _config.IconNoInternet;
      }
    }

    private string assignWifiIcon(int signalQuality)
    {
      // Round to nearest multiple of 25
      return ((signalQuality % 25) > (25 / 2) ? (signalQuality / 25) + 1 : signalQuality / 25) switch
      {
        0 => _config.IconWifiSignal0,
        1 => _config.IconWifiSignal25,
        2 => _config.IconWifiSignal50,
        3 => _config.IconWifiSignal75,
        4 => _config.IconWifiSignal100,
        _ => _config.IconNoInternet,
      };
    }

    public NetworkComponentViewModel(
      BarViewModel parentViewModel,
      NetworkComponentConfig config) : base(parentViewModel, config)
    {
      Observable.Interval(TimeSpan.FromSeconds(10))
        .Subscribe(_ => OnPropertyChanged(nameof(Text)));
      NetworkChange.NetworkAddressChanged += new NetworkAddressChangedEventHandler((s, e) => OnPropertyChanged(nameof(Text)));
    }
  }
}
