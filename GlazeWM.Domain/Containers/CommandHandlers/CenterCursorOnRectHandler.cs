using System.Diagnostics;
using System.Net.NetworkInformation;
using System.Text;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.Bussing;
using ManagedNativeWifi;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;
using PInvoke;
using static PInvoke.IPHlpApi;
using System;
using System.Linq;
using static Vanara.PInvoke.WlanApi;
using static Vanara.PInvoke.IpHlpApi;
using static Vanara.PInvoke.Ws2_32;
using Vanara.Extensions;
using Vanara.InteropServices;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class CenterCursorOnRectHandler : ICommandHandler<CenterCursorOnRectCommand>
  {
    private readonly UserConfigService _userConfigService;

    public CenterCursorOnRectHandler(UserConfigService userConfigService)
    {
      _userConfigService = userConfigService;
    }

    public CommandResponse Handle(CenterCursorOnRectCommand command)
    {
      var isEnabled = _userConfigService.GeneralConfig.CursorFollowsFocus;

      if (!isEnabled)
        return CommandResponse.Ok;

      var targetRect = command.TargetRect;

      // Calculate center point of focused window.
      var centerX = targetRect.X + (targetRect.Width / 2);
      var centerY = targetRect.Y + (targetRect.Height / 2);

      SetCursorPos(centerX, centerY);


      // "40:E1:E4:63:72:C5"

      var bssNetworks = NativeWifi.EnumerateBssNetworks();
      var connectedSSID = NativeWifi.EnumerateConnectedNetworkSsids().FirstOrDefault()?.ToString();
      var connectedInterface = NativeWifi.EnumerateInterfaceConnections();

      // Get the index of the interface with the best route to the remote address.
      var dwDestAddr = System.BitConverter.ToUInt32(Encoding.ASCII.GetBytes("8.8.8.8"));
      uint dwBestIfIndex = 0;
      // GetBestInterface(dwDestAddr, ref dwBestIfIndex)
      var result = GetBestInterface(dwDestAddr, out dwBestIfIndex);

      var primaryAdapter = GetAdaptersAddresses(GetAdaptersAddressesFlags.GAA_FLAG_INCLUDE_GATEWAYS).FirstOrDefault(r => r.OperStatus == IF_OPER_STATUS.IfOperStatusUp && r.TunnelType == TUNNEL_TYPE.TUNNEL_TYPE_NONE && r.FirstGatewayAddress != IntPtr.Zero);
      var primaryIndex = primaryAdapter.IfIndex;

      var hWlan = WlanOpenHandle();


      WlanEnumInterfaces(hWlan, default, out var list).ThrowIfFailed();
      if (list.dwNumberOfItems < 1)
        throw new InvalidOperationException("No WLAN interfaces.");
      var intf = list.InterfaceInfo[0].InterfaceGuid;
      var conn = list.InterfaceInfo[0].isState == WLAN_INTERFACE_STATE.wlan_interface_state_connected;
      Debug.WriteLine(intf.ToString());

      var q = WlanGetNetworkBssList(hWlan, intf, IntPtr.Zero, DOT11_BSS_TYPE.dot11_BSS_type_any, true, default, out var mem);
      var elist = mem.DangerousGetHandle().ToStructure<WLAN_BSS_LIST>();
      var x = WlanGetAvailableNetworkList(hWlan, intf, 3, default, out var listz);
      var z = WlanGetInterfaceCapability(hWlan, intf, default, out var listzz);


      var ee = WlanQueryInterface(hWlan, intf, WLAN_INTF_OPCODE.wlan_intf_opcode_radio_state, default, out var sz, out var data, out var type);
      MIB_IPADDRTABLE t = GetIpAddrTable();

      uint len = 15000;
      var memm = new SafeCoTaskMemHandle((int)len);
      var xy = GetPerAdapterInfo(primaryAdapter.IfIndex);


      // Find a matching .NET interface object with the given index.
      foreach (var networkInterface in NetworkInterface.GetAllNetworkInterfaces())
        if (networkInterface.GetIPProperties().GetIPv4Properties().Index == dwBestIfIndex)
        {
          var y = bssNetworks.Where(x => x.Interface.Id.ToString() == networkInterface.Id);

        }

      return CommandResponse.Ok;
    }
  }
}
