using System.Diagnostics;
using System.Linq;
using System.Net.NetworkInformation;
using System.Windows.Input;
using GlazeWM.Bar.Common;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;
using ManagedNativeWifi;

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
      foreach (var adapter in NetworkInterface.GetAllNetworkInterfaces())
      {
        if (adapter.OperationalStatus == OperationalStatus.Up && adapter.NetworkInterfaceType != NetworkInterfaceType.Loopback)
        {
          Debug.WriteLine(adapter.NetworkInterfaceType);
          if (adapter.NetworkInterfaceType == NetworkInterfaceType.Ethernet)
          {
            Debug.WriteLine("here");
          }
        }
      }

      var bssNetworks = NativeWifi.EnumerateBssNetworks();
      var allC = NativeWifi.EnumerateConnectedNetworkSsids();
      var connectedSSID = NativeWifi.EnumerateConnectedNetworkSsids().FirstOrDefault();
      var linkQuality = bssNetworks.Where(network => network?.Ssid.ToString() == connectedSSID.ToString())?.FirstOrDefault()?.LinkQuality.ToString();
      _config.Text = linkQuality != null ? linkQuality : "Eth";
      // _config.Text = "";



      //   var availableNetwork = NativeWifi.EnumerateAvailableNetworks()
      //   .Where(x => !string.IsNullOrWhiteSpace(x.ProfileName))
      //   .OrderByDescending(x => x.SignalQuality)
      //   .FirstOrDefault();

      //   var bssNetworks = NativeWifi.EnumerateBssNetworks();


      //   // "{1B243423-099E-423F-8500-E5785E026467}"

      //   var currentNetwork = NativeWifi.EnumerateConnectedNetworkSsids().FirstOrDefault();
      //   // var currentNetworkInfo = NativeWifi.GetInterfaceRadio(currentNetwork.Id);
      //   var allNetworks = NativeWifi.EnumerateAvailableNetworks();
      //   var connectedNetworkDetails = NativeWifi.EnumerateAvailableNetworks()
      //     .FirstOrDefault(x => x.Ssid.ToString() == currentNetwork.ToString());

      //   NetworkChange.NetworkAddressChanged += (object sender, System.EventArgs e) =>
      // {
      //   Debug.WriteLine(e);
      //   Debug.WriteLine("connected network changed !");
      // };

      //https://stackoverflow.com/questions/25303847/rssi-using-windows-api

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
