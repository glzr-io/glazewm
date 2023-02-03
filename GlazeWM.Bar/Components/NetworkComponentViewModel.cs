using System;
using System.Linq;
using System.Reactive.Linq;
using System.Text;
using GlazeWM.Domain.UserConfigs;
using Vanara.InteropServices;
using static Vanara.PInvoke.IpHlpApi;
using static Vanara.PInvoke.WlanApi;

namespace GlazeWM.Bar.Components
{
  public class NetworkComponentViewModel : ComponentViewModel
  {
    private NetworkComponentConfig _config => _componentConfig as NetworkComponentConfig;
    public string Text => FormatLabel();
    public string _iconText;
    public string IconText
    {
      get => _iconText;
      set
      {
        _iconText = value;
        OnPropertyChanged(nameof(IconText));
      }
    }
    public string IconFontFamily => _config.IconFontFamily;

    private string currentSSID = "";
    private string currentSignalQuality = "";
    private string FormatLabel()
    {
      IconText = getNetworkIcon();
      return currentSSID + "/" + currentSignalQuality;
    }

    private string getNetworkIcon()
    {
      var primaryAdapterID = getPrimaryAdapterID();

      var primaryAdapter = GetAdaptersAddresses(GetAdaptersAddressesFlags.GAA_FLAG_INCLUDE_GATEWAYS).FirstOrDefault(
          r => r.OperStatus == IF_OPER_STATUS.IfOperStatusUp
          && r.TunnelType == TUNNEL_TYPE.TUNNEL_TYPE_NONE
          && r.FirstGatewayAddress != IntPtr.Zero
          && r.IfIndex == primaryAdapterID
        );

      switch (primaryAdapter.IfType)
      {
        case IFTYPE.IF_TYPE_ETHERNET_CSMACD:
        case IFTYPE.IF_TYPE_ETHERNET_3MBIT:
          // Primary adapter is using ethernet.
          return _config.IconEthernet
          ;
        case IFTYPE.IF_TYPE_IEEE80211:
          // Primary adapter is using wifi, find the primary WLAN interface
          var hWlan = WlanOpenHandle();
          WlanEnumInterfaces(hWlan, default, out var list);
          if (list == null || list.dwNumberOfItems < 1)
            return _config.IconNoInternet;
          var primaryIntfGuid = list.InterfaceInfo[0].InterfaceGuid;

          // Get RSSI and wifi connection details
          var getType = CorrespondingTypeAttribute.GetCorrespondingTypes(WLAN_INTF_OPCODE.wlan_intf_opcode_current_connection, CorrespondingAction.Get).FirstOrDefault();
          var interfaceDetails = WlanQueryInterface(hWlan, primaryIntfGuid, WLAN_INTF_OPCODE.wlan_intf_opcode_current_connection, default, out var dataSize, out var data, out var type);
          if (interfaceDetails.Failed)
            return _config.IconNoInternet;

          var connectionAttributes = (WLAN_CONNECTION_ATTRIBUTES)data.DangerousGetHandle().Convert(dataSize, getType);
          var signalQuality = connectionAttributes.wlanAssociationAttributes.wlanSignalQuality;
          currentSignalQuality = signalQuality.ToString();
          currentSSID = connectionAttributes.strProfileName;
          return assignWifiIcon(signalQuality);
        default:
          return _config.IconNoInternet;
      }

    }

    private static uint getPrimaryAdapterID()
    {
      // Get primary adapter using Google DNS as example IP for IP to check against.   
      var dwDestAddr = BitConverter.ToUInt32(Encoding.ASCII.GetBytes("8.8.8.8"));
      GetBestInterface(dwDestAddr, out var dwBestIfIndex);
      return dwBestIfIndex;
    }

    private string assignWifiIcon(uint signalQuality)
    {
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
    }
  }
}
