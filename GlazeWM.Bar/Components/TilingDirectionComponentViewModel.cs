using System;
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

    private TilingDirectionComponentConfig _config => _componentConfig as TilingDirectionComponentConfig;

    private string LabelVertical => _config.LabelVertical;
    private string LabelHorizontal => _config.LabelHorizontal;

    /// <summary>
    /// The tiling direction of the currently focused container. Can be null on app
    /// startup when workspaces haven't been created yet.
    /// </summary>
    private TilingDirection? _tilingDirection =>
      (_containerService.FocusedContainer as SplitContainer)?.TilingDirection ??
      (_containerService.FocusedContainer.Parent as SplitContainer)?.TilingDirection;

    public string TilingDirectionString =>
      _tilingDirection == TilingDirection.Vertical ? LabelVertical : LabelHorizontal;

    public TilingDirectionComponentViewModel(
      BarViewModel parentViewModel,
      TilingDirectionComponentConfig config) : base(parentViewModel, config)
    {
      _bus.Events.Where(
        (@event) => @event is TilingDirectionChangedEvent or FocusChangedEvent
      ).Subscribe((_) =>
        _dispatcher.Invoke(() => OnPropertyChanged(nameof(TilingDirectionString)))
      );
    }
  }
}
