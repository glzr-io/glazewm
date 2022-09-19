using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure;

namespace GlazeWM.Domain.Containers
{
  public class SplitContainer : Container, IResizable
  {
    public Layout Layout { get; set; } = Layout.HORIZONTAL;

    public double SizePercentage { get; set; } = 1;

    private readonly ContainerService _containerService =
      ServiceLocator.GetRequiredService<ContainerService>();

    public override int Width => _containerService.GetWidthOfResizableContainer(this);
    public override int Height => _containerService.GetHeightOfResizableContainer(this);
    public override int X => _containerService.GetXOfResizableContainer(this);
    public override int Y => _containerService.GetYOfResizableContainer(this);
  }
}
