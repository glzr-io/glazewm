using System;
using System.Collections.Generic;
using System.Reactive.Linq;
using GlazeWM.Bar;
using GlazeWM.Domain.Common.Commands;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Commands;
using GlazeWM.Infrastructure.Common.Events;
using GlazeWM.Infrastructure.WindowsApi;
using GlazeWM.Application.IpcServer;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Application.WM
{
  public sealed class WindowManager
  {
    private readonly BarService _barService;
    private readonly Bus _bus;
    private readonly KeybindingService _keybindingService;
    private readonly WindowEventService _windowEventService;
    private readonly UserConfigService _userConfigService;
    private readonly IpcServerManager _ipcServerManager;

    private SystemTrayIcon _systemTrayIcon { get; set; }

    public Startup(
      BarService barService,
      Bus bus,
      KeybindingService keybindingService,
      WindowEventService windowEventService,
      UserConfigService userConfigService,
      IpcServerManager ipcServerManager)
    {
      _barService = barService;
      _bus = bus;
      _keybindingService = keybindingService;
      _windowEventService = windowEventService;
      _userConfigService = userConfigService;
      _ipcServerManager = ipcServerManager;
    }

    public void Start()
    {
      try
      {
        // Set the process-default DPI awareness.
        _ = SetProcessDpiAwarenessContext(DpiAwarenessContext.PerMonitorAwareV2);

        _bus.Events.OfType<ApplicationExitingEvent>()
          .Subscribe(_ => OnApplicationExit());

        _bus.Events.OfType<FocusChangedEvent>().Subscribe((@event) => _bus.InvokeAsync(new SetActiveWindowBorderCommand(@event.FocusedContainer as Window)));

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
          IconResourceName = "GlazeWM.Application.Resources.icon.ico",
          Actions = new Dictionary<string, Action>
          {
            { "Reload config", () => _bus.Invoke(new ReloadUserConfigCommand()) },
            { "Exit", () => _bus.Invoke(new ExitApplicationCommand(false)) },
          }
        };

        // Add application to system tray.
        _systemTrayIcon = new SystemTrayIcon(systemTrayIconConfig);
        _systemTrayIcon.Show();

        // Hook mouse event for focus follows cursor.
        if (_userConfigService.GeneralConfig.FocusFollowsCursor)
          MouseEvents.MouseMoves.Sample(TimeSpan.FromMilliseconds(50)).Subscribe((@event) =>
          {
            if (!@event.IsMouseDown)
              _bus.InvokeAsync(new FocusContainerUnderCursorCommand(@event.Point));
          });

        System.Windows.Forms.Application.Run();
      }
      catch (Exception exception)
      {
        _bus.Invoke(new HandleFatalExceptionCommand(exception));
      }
    }

    private void OnApplicationExit()
    {
      _bus.Invoke(new ShowAllWindowsCommand());
      _bus.Invoke(new SetActiveWindowBorderCommand(null));
      _barService.ExitApp();
      _systemTrayIcon?.Remove();
      _ipcServerManager.StopIpcServer();
      System.Windows.Forms.Application.Exit();
    }
  }
}
