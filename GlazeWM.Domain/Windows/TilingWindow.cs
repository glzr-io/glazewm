using System;
using GlazeWM.Domain.Containers;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.WindowsApi;
using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Domain.Windows
{
  public sealed class TilingWindow : Window, IResizable
  {
    public double SizePercentage { get; set; } = 1;

    private readonly ContainerService _containerService =
      ServiceLocator.Provider.GetRequiredService<ContainerService>();

    public override int Width => _containerService.CalculateWidthOfResizableContainer(this);

    public override int Height => _containerService.CalculateHeightOfResizableContainer(this);

    public override int X => _containerService.CalculateXOfResizableContainer(this);

    public override int Y => _containerService.CalculateYOfResizableContainer(this);

    public TilingWindow(
      IntPtr handle,
      WindowRect floatingPlacement,
      RectDelta borderDelta
    ) : base(handle, floatingPlacement, borderDelta)
    {
    }

    public TilingWindow(
      IntPtr handle,
      WindowRect floatingPlacement,
      RectDelta borderDelta,
      double sizePercentage
    ) : base(handle, floatingPlacement, borderDelta)
    {
      SizePercentage = sizePercentage;
    }
  }
}
