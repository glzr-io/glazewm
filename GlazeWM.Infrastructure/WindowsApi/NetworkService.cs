using System;
using System.Linq;
using System.Net.NetworkInformation;
using System.Text;
using Vanara.InteropServices;
using static Vanara.PInvoke.IpHlpApi;
using static Vanara.PInvoke.WlanApi;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public static class NetworkService
  {
    public static int GetWlanRSSI()
    {
      var hWlan = WlanOpenHandle();
      WlanEnumInterfaces(hWlan, default, out var list);
      if (list == null || list.dwNumberOfItems < 1)
        return -1;
      var primaryIntfGuid = list.InterfaceInfo[0].InterfaceGuid;

      // Get wireless connection details.
      var getType = CorrespondingTypeAttribute.GetCorrespondingTypes(WLAN_INTF_OPCODE.wlan_intf_opcode_current_connection, CorrespondingAction.Get).FirstOrDefault();
      var interfaceDetails = WlanQueryInterface(hWlan, primaryIntfGuid, WLAN_INTF_OPCODE.wlan_intf_opcode_current_connection, default, out var dataSize, out var data, out _);
      if (interfaceDetails.Failed)
        return -1;

      // Get RSSI signal strength of connection.  
      var connectionAttributes = (WLAN_CONNECTION_ATTRIBUTES)data.DangerousGetHandle().Convert(dataSize, getType);
      return (int)connectionAttributes.wlanAssociationAttributes.wlanSignalQuality;
    }

    public static bool PingTest()
    {
      try
      {
        // Use Google DNS servers to check if online.
        var reply = new Ping().Send("8.8.8.8");
        return reply.Status == IPStatus.Success;
      }
      catch (Exception)
      {
        return false;
      }
    }

    public static uint GetPrimaryAdapterID()
    {
      // Using Google DNS as example IP for IP to check against.   
      var dwDestAddr = BitConverter.ToUInt32(Encoding.ASCII.GetBytes("8.8.8.8"));
      GetBestInterface(dwDestAddr, out var dwBestIfIndex);
      return dwBestIfIndex;
    }

    public static IP_ADAPTER_ADDRESSES GetPrimaryAdapter()
    {
      // Filter out active tunnels and VPNS.
      return GetAdaptersAddresses(GetAdaptersAddressesFlags.GAA_FLAG_INCLUDE_GATEWAYS).FirstOrDefault(
        r => r.OperStatus == IF_OPER_STATUS.IfOperStatusUp
        && r.TunnelType == TUNNEL_TYPE.TUNNEL_TYPE_NONE
        && r.FirstGatewayAddress != IntPtr.Zero
        && r.IfIndex == GetPrimaryAdapterID()
        );
    }
  }
}
