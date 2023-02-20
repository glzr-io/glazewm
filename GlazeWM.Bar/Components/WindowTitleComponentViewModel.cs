using System;
using System.Reactive.Linq;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common.Events;

namespace GlazeWM.Bar.Components
{
  public class WindowTitleComponentViewModel : ComponentViewModel
  {
    private readonly Bus _bus = ServiceLocator.GetRequiredService<Bus>();
    private readonly ContainerService _containerService =
      ServiceLocator.GetRequiredService<ContainerService>();

    public string _focusedWindowTitle = string.Empty;
    public string FocusedWindowTitle
    {
      get => _focusedWindowTitle;
      set
      {
        _focusedWindowTitle = value;
        OnPropertyChanged(nameof(FocusedWindowTitle));
      }
    }

    public WindowTitleComponentViewModel(
      BarViewModel parentViewModel,
      WindowTitleComponentConfig config) : base(parentViewModel, config)
    {
      var windowFocusedSubscription = _bus.Events.OfType<WindowFocusedEvent>()
        .Subscribe((@event) => UpdateTitle(@event.WindowHandle));

      var windowTitleChangedSubscription = _bus.Events.OfType<WindowTitleChangedEvent>()
        .Subscribe((@event) => UpdateTitle(@event.WindowHandle));

      RegisterDisposables(windowFocusedSubscription, windowTitleChangedSubscription);
    }

    private void UpdateTitle(IntPtr windowHandle)
    {
      var focusedWindow = _containerService.FocusedContainer as Window;

      if (focusedWindow != null && windowHandle != focusedWindow.Handle)
        return;

      // TODO: Make truncate max length configurable from config.
      var windowTitle = focusedWindow?.Title ?? string.Empty;
      FocusedWindowTitle = Truncate(windowTitle, 60);
    }

    public static string Truncate(string value, int maxLength, string truncationSuffix = "…")
    {
      return value?.Length > maxLength
        ? string.Concat(value.AsSpan(0, maxLength), truncationSuffix)
        : value;
    }
  }
}
