using System.Diagnostics;
using System.Windows.Input;
using GlazeWM.Bar.Common;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;

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
      // ShellConfig shellConfig = ShellManager.DefaultShellConfig;

      // // Customize tray service options.
      // shellConfig.EnableTrayService = true; // controls whether the tray objects are instantiated in ShellManager, true by default
      // shellConfig.AutoStartTrayService = false; // controls whether the tray service is started as part of ShellManager instantiation, true by default
      // shellConfig.PinnedNotifyIcons = new[] { "GUID or PathToExe:UID" }; // sets the initial NotifyIcons that should be included in the PinnedIcons collection, by default Action Center (prior to Windows 10 only), Power, Network, and Volume.

      // // Initialize the shell manager.
      // ShellManager _shellManager = new ShellManager(shellConfig);

      // // Initialize the tray service, since we disabled auto-start above.
      // _shellManager.NotificationArea.Initialize();
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
