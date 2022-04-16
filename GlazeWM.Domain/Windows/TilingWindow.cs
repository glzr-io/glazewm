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
      IntPtr hwnd,
      WindowRect floatingPlacement,
      RectDelta borderDelta
    ) : base(hwnd, floatingPlacement, borderDelta)
    {
    }

    public TilingWindow(
      IntPtr hwnd,
      WindowRect floatingPlacement,
      RectDelta borderDelta,
      double sizePercentage
    ) : base(hwnd, floatingPlacement, borderDelta)
    {
      SizePercentage = sizePercentage;
    }
  }
}
