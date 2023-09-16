using System;
using System.Collections.Generic;
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
    private readonly WindowTitleComponentConfig _config;
    private readonly Bus _bus = ServiceLocator.GetRequiredService<Bus>();
    private readonly ContainerService _containerService =
      ServiceLocator.GetRequiredService<ContainerService>();

    private LabelViewModel _label;
    public LabelViewModel Label
    {
      get => _label;
      protected set => SetField(ref _label, value);
    }

    public WindowTitleComponentViewModel(
      BarViewModel parentViewModel,
      WindowTitleComponentConfig config) : base(parentViewModel, config)
    {
      _config = config;

      var windowFocused = _bus.Events
        .OfType<WindowFocusedEvent>()
        .Select(@event => @event.WindowHandle);

      var windowTitleChanged = _bus.Events
        .OfType<WindowTitleChangedEvent>()
        .Select(@event => @event.WindowHandle);

      windowFocused.Merge(windowTitleChanged)
        .TakeUntil(_parentViewModel.WindowClosing)
        .Subscribe((windowHandle) =>
        {
          var focusedWindow = _containerService.FocusedContainer as Window;

          if (focusedWindow != null && windowHandle != focusedWindow.Handle)
            return;

          var windowTitle = focusedWindow?.Title ?? string.Empty;
          Label = CreateLabel(windowTitle);
        });
    }

    private LabelViewModel CreateLabel(string windowTitle)
    {
      var variableDictionary = new Dictionary<string, Func<string>>()
      {
      // TODO: Make truncate max length configurable from config.
        { "window_title", () => Truncate(windowTitle, 60) }
      };

      return XamlHelper.ParseLabel(_config.Label, variableDictionary, this);
    }

    public static string Truncate(string value, int maxLength, string truncationSuffix = "…")
    {
      return value?.Length > maxLength
        ? string.Concat(value.AsSpan(0, maxLength), truncationSuffix)
        : value;
    }
  }
}
