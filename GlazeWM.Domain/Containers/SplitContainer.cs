using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure;
using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Domain.Containers
{
  public class SplitContainer : Container, IResizable
  {
    public Layout Layout { get; set; } = Layout.HORIZONTAL;

    public double SizePercentage { get; set; } = 1;

    private readonly ContainerService _containerService =
      ServiceLocator.Provider.GetRequiredService<ContainerService>();

    public override int Width => _containerService.CalculateWidthOfResizableContainer(this);
    public override int Height => _containerService.CalculateHeightOfResizableContainer(this);
    public override int X => _containerService.CalculateXOfResizableContainer(this);
    public override int Y => _containerService.CalculateYOfResizableContainer(this);
  }
}
