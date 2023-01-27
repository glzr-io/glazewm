using System.Diagnostics;
using System.Linq;
using System.Net.NetworkInformation;
using System.Windows.Input;
using GlazeWM.Bar.Common;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;
using static Vanara.PInvoke.WlanApi;
using static Vanara.PInvoke.IpHlpApi;
using static Vanara.PInvoke.Ws2_32;
using System;
using System.Text;
using Vanara.InteropServices;

namespace GlazeWM.Bar.Components
{
  public class NetworkComponentViewModel : ComponentViewModel
  {
    private readonly Bus _bus = ServiceLocator.GetRequiredService<Bus>();
    private readonly ContainerService _containerService =
      ServiceLocator.GetRequiredService<ContainerService>();
    private readonly CommandParsingService _commandParsingService =
      ServiceLocator.GetRequiredService<CommandParsingService>();
    private NetworkComponentConfig _config => _componentConfig as NetworkComponentConfig;

    public string Text => _config.Text;

    public ICommand LeftClickCommand => new RelayCommand(OnLeftClick);
    public ICommand RightClickCommand => new RelayCommand(OnRightClick);

    public NetworkComponentViewModel(
      BarViewModel parentViewModel,
      NetworkComponentConfig config) : base(parentViewModel, config)
    {
      var dwDestAddr = BitConverter.ToUInt32(Encoding.ASCII.GetBytes("8.8.8.8"));
      GetBestInterface(dwDestAddr, out var dwBestIfIndex);
      var primaryAdapter = GetAdaptersAddresses(GetAdaptersAddressesFlags.GAA_FLAG_INCLUDE_GATEWAYS).FirstOrDefault(
          r => r.OperStatus == IF_OPER_STATUS.IfOperStatusUp
          && r.TunnelType == TUNNEL_TYPE.TUNNEL_TYPE_NONE
          && r.FirstGatewayAddress != IntPtr.Zero
          && r.IfIndex == dwBestIfIndex
        );

      switch (primaryAdapter.IfType)
      {
        case IFTYPE.IF_TYPE_ETHERNET_CSMACD:
        case IFTYPE.IF_TYPE_ETHERNET_3MBIT:
          Debug.WriteLine("HEREEE");
          break;
        case IFTYPE.IF_TYPE_IEEE80211:
          var hWlan = WlanOpenHandle();

          WlanEnumInterfaces(hWlan, default, out var list).ThrowIfFailed();
          if (list.dwNumberOfItems < 1)
            throw new InvalidOperationException("No WLAN interfaces.");
          var intf = list.InterfaceInfo[0].InterfaceGuid;

          var getType = CorrespondingTypeAttribute.GetCorrespondingTypes(WLAN_INTF_OPCODE.wlan_intf_opcode_current_connection, CorrespondingAction.Get).FirstOrDefault();
          var ee = WlanQueryInterface(hWlan, intf, WLAN_INTF_OPCODE.wlan_intf_opcode_current_connection, default, out var sz, out var data, out var type);
          if (ee.Failed)
            break;
          var yyy = (WLAN_CONNECTION_ATTRIBUTES)data.DangerousGetHandle().Convert(sz, getType);
          var sigQual = yyy.wlanAssociationAttributes.wlanSignalQuality;
          break;
      }

      bool pingable = false;
      Ping pinger = null;
      try
      {
        pinger = new Ping();
        PingReply reply = pinger.Send("8.8.8.8");
        pingable = reply.Status == IPStatus.Success;
      }
      catch (PingException)
      {
        // Discard PingExceptions and return false;
      }
      finally
      {
        if (pinger != null)
        {
          pinger.Dispose();
        }
      }
      Debug.WriteLine("--");
    }

    public void OnLeftClick()
    {
      InvokeCommand(_config.LeftClickCommand);
    }

    public void OnRightClick()
    {
      InvokeCommand(_config.RightClickCommand);
    }

    private void InvokeCommand(string commandString)
    {
      if (string.IsNullOrEmpty(commandString))
        return;

      var subjectContainer = _containerService.FocusedContainer;

      var parsedCommand = _commandParsingService.ParseCommand(
        commandString,
        subjectContainer
      );

      // Use `dynamic` to resolve the command type at runtime and allow multiple dispatch.
      _bus.Invoke((dynamic)parsedCommand);
    }
  }
}
