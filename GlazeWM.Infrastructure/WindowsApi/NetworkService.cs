using Vanara.InteropServices;
using static Vanara.PInvoke.WlanApi;
using static Vanara.PInvoke.IpHlpApi;
using System;
using System.Text;
using System.Net.NetworkInformation;
using System.Linq;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public class NetworkService
  {
    public static int getWlanRSSI()
    {
      var hWlan = WlanOpenHandle();
      WlanEnumInterfaces(hWlan, default, out var list);
      if (list == null || list.dwNumberOfItems < 1)
        return -1;
      var primaryIntfGuid = list.InterfaceInfo[0].InterfaceGuid;

      // Get wireless connection details.
      var getType = CorrespondingTypeAttribute.GetCorrespondingTypes(WLAN_INTF_OPCODE.wlan_intf_opcode_current_connection, CorrespondingAction.Get).FirstOrDefault();
      var interfaceDetails = WlanQueryInterface(hWlan, primaryIntfGuid, WLAN_INTF_OPCODE.wlan_intf_opcode_current_connection, default, out var dataSize, out var data, out var type);
      if (interfaceDetails.Failed)
        return -1;

      //   _currentSSID = connectionAttributes.strProfileName;

      // Get RSSI signal strength of connection.  
      var connectionAttributes = (WLAN_CONNECTION_ATTRIBUTES)data.DangerousGetHandle().Convert(dataSize, getType);
      return (int)connectionAttributes.wlanAssociationAttributes.wlanSignalQuality;
    }

    public static bool pingTest()
    {
      var pingable = false;
      Ping pinger = null;
      try
      {
        pinger = new Ping();
        // Use Google DNS servers to check if online.
        var reply = pinger.Send("8.8.8.8");
        pingable = reply.Status == IPStatus.Success;
      }
      catch (PingException)
      {
        return false;
      }
      finally
      {
        pinger?.Dispose();
      }
      return pingable;
    }

    public static uint getPrimaryAdapterID()
    {
      // Using Google DNS as example IP for IP to check against.   
      var dwDestAddr = BitConverter.ToUInt32(Encoding.ASCII.GetBytes("8.8.8.8"));
      GetBestInterface(dwDestAddr, out var dwBestIfIndex);
      return dwBestIfIndex;
    }

    public static IP_ADAPTER_ADDRESSES getPrimaryAdapter()
    {
      // Filter out active tunnels and VPNS.
      return GetAdaptersAddresses(GetAdaptersAddressesFlags.GAA_FLAG_INCLUDE_GATEWAYS).FirstOrDefault(
        r => r.OperStatus == IF_OPER_STATUS.IfOperStatusUp
        && r.TunnelType == TUNNEL_TYPE.TUNNEL_TYPE_NONE
        && r.FirstGatewayAddress != IntPtr.Zero
        && r.IfIndex == getPrimaryAdapterID()
        );
    }
  }
}
