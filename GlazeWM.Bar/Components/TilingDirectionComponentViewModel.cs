using System;
using System.Collections.Generic;
using System.Reactive.Linq;
using System.Windows.Threading;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Bar.Components
{
  public class TilingDirectionComponentViewModel : ComponentViewModel
  {
    private Dispatcher _dispatcher => _parentViewModel.Dispatcher;
    private readonly Bus _bus = ServiceLocator.GetRequiredService<Bus>();
    private readonly ContainerService _containerService =
      ServiceLocator.GetRequiredService<ContainerService>();

    private readonly TilingDirectionComponentConfig _config;

    private LabelViewModel _label;
    public LabelViewModel Label
    {
      get => _label;
      protected set => SetField(ref _label, value);
    }

    public TilingDirectionComponentViewModel(
      BarViewModel parentViewModel,
      TilingDirectionComponentConfig config) : base(parentViewModel, config)
    {
      _config = config;

      _bus.Events.Where(
        (@event) => @event is TilingDirectionChangedEvent or FocusChangedEvent
      )
        .TakeUntil(_parentViewModel.WindowClosing)
        .Subscribe((_) => _dispatcher.Invoke(() => Label = CreateLabel()));
    }

    private LabelViewModel CreateLabel()
    {
      // The tiling direction of the currently focused container. Can be null on app
      // startup when workspaces haven't been created yet.
      var tilingDirection =
        (_containerService.FocusedContainer as SplitContainer)?.TilingDirection ??
        (_containerService.FocusedContainer.Parent as SplitContainer)?.TilingDirection;

      var label = tilingDirection == TilingDirection.Vertical
        ? _config.LabelVertical
        : _config.LabelHorizontal;

      return XamlHelper.ParseLabel(label, new Dictionary<string, Func<string>>(), this);
    }
  }
}
