using GlazeWM.Domain.Common;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure;

namespace GlazeWM.Domain.Containers
{
  public class SplitContainer : Container, IResizable
  {
    /// <inheritdoc />
    public override ContainerType Type { get; } = ContainerType.Split;

    public TilingDirection TilingDirection { get; set; } = TilingDirection.Horizontal;

    public double SizePercentage { get; set; } = 1;

    private readonly ContainerService _containerService =
      ServiceLocator.GetRequiredService<ContainerService>();

    public override int Width => _containerService.GetWidthOfResizableContainer(this);
    public override int Height => _containerService.GetHeightOfResizableContainer(this);
    public override int X => _containerService.GetXOfResizableContainer(this);
    public override int Y => _containerService.GetYOfResizableContainer(this);
  }
}
