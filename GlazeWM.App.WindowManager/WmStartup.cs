using System.Reactive.Linq;
using GlazeWM.Domain.Common.Commands;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common;
using GlazeWM.Infrastructure.Common.Commands;
using GlazeWM.Infrastructure.Common.Events;
using GlazeWM.Infrastructure.WindowsApi;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.App.WindowManager
{
  public sealed class WmStartup
  {
    private readonly Bus _bus;
    private readonly KeybindingService _keybindingService;
    private readonly WindowEventService _windowEventService;
    private readonly UserConfigService _userConfigService;

    private SystemTrayIcon? _systemTrayIcon { get; set; }

    public WmStartup(
      Bus bus,
      KeybindingService keybindingService,
      WindowEventService windowEventService,
      UserConfigService userConfigService)
    {
      _bus = bus;
      _keybindingService = keybindingService;
      _windowEventService = windowEventService;
      _userConfigService = userConfigService;
    }

    public ExitCode Run()
    {
      try
      {
        // Set the process-default DPI awareness.
        _ = SetProcessDpiAwarenessContext(DpiAwarenessContext.PerMonitorAwareV2);

        _bus.Events.OfType<ApplicationExitingEvent>()
          .Subscribe(_ => OnApplicationExit());

        // Populate initial monitors, windows, workspaces and user config.
        _bus.Invoke(new PopulateInitialStateCommand());

        // Listen on registered keybindings.
        _keybindingService.Start();

        // Listen for window events (eg. close, focus).
        _windowEventService.Start();

        // Listen for changes to display settings.
        // TODO: Unsubscribe on application exit.
        SystemEvents.DisplaySettingsChanged.Subscribe((@event) => _bus.EmitAsync(@event));

        var systemTrayIconConfig = new SystemTrayIconConfig
        {
          HoverText = "GlazeWM",
          IconResourceName = "GlazeWM.App.Resources.icon.ico",
          Actions = new Dictionary<string, Action>
          {
            { "Reload config", () => _bus.Invoke(new ReloadUserConfigCommand()) },
            { "Exit", () => _bus.Invoke(new ExitApplicationCommand(false)) },
          }
        };

        // Add application to system tray.
        _systemTrayIcon = new SystemTrayIcon(systemTrayIconConfig);
        _systemTrayIcon.Show();

        var nativeFocusReassigned = _bus.Events
          .OfType<NativeFocusReassignedEvent>()
          .Select((@event) => @event.FocusedContainer);

        if (_userConfigService.FocusBorderConfig.Active.Enabled ||
            _userConfigService.FocusBorderConfig.Inactive.Enabled)
        {
          var focusChanged = _bus.Events
            .OfType<FocusChangedEvent>()
            .Select(@event => @event.FocusedContainer);

          focusChanged.Merge(nativeFocusReassigned)
            .Where(container => container is Window)
            .Subscribe((window) => _bus.InvokeAsync(new SetActiveWindowBorderCommand(window as Window)));
        }

        // Hook mouse event for focus follows cursor.
        if (_userConfigService.GeneralConfig.FocusFollowsCursor)
          MouseEvents.MouseMoves.Sample(TimeSpan.FromMilliseconds(50)).Subscribe((@event) =>
          {
            if (!@event.IsLMouseDown && !@event.IsRMouseDown)
              _bus.InvokeAsync(new FocusContainerUnderCursorCommand(@event.Point));
          });

        // Setup cursor follows focus
        if (_userConfigService.GeneralConfig.CursorFollowsFocus)
        {
          var focusedContainerMoved = _bus.Events
            .OfType<FocusedContainerMovedEvent>()
            .Select(@event => @event.FocusedContainer);

          focusedContainerMoved.Merge(nativeFocusReassigned)
            .Where(container => container is Window)
            .Subscribe((window) => _bus.InvokeAsync(new CenterCursorOnContainerCommand(window)));
        }

        System.Windows.Forms.Application.Run();
        return ExitCode.Success;
      }
      catch (Exception exception)
      {
        _bus.Invoke(new HandleFatalExceptionCommand(exception));
        return ExitCode.Error;
      }
    }

    private void OnApplicationExit()
    {
      _bus.Invoke(new ShowAllWindowsCommand());
      _bus.Invoke(new SetActiveWindowBorderCommand(null));
      _systemTrayIcon?.Remove();
      System.Windows.Forms.Application.Exit();
    }
  }
}
