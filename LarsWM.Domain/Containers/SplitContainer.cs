using LarsWM.Domain.Common.Enums;
using LarsWM.Infrastructure;
using Microsoft.Extensions.DependencyInjection;

namespace LarsWM.Domain.Containers
{
  public class SplitContainer : Container
  {
    public Layout Layout { get; set; } = Layout.HORIZONTAL;

    public override int Width => _containerService.CalculateWidthOfResizableContainer(this);

    public override int Height => _containerService.CalculateHeightOfResizableContainer(this);

    public override int X => _containerService.CalculateXOfResizableContainer(this);

    public override int Y => _containerService.CalculateYOfResizableContainer(this);

    private ContainerService _containerService = ServiceLocator.Provider.GetRequiredService<ContainerService>();
  }
}
