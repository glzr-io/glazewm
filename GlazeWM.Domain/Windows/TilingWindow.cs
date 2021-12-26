using System;
using System.Drawing;
using GlazeWM.Domain.Containers;
using GlazeWM.Infrastructure;
using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Domain.Windows
{
  public sealed class TilingWindow : Window, IResizable
  {
    public double SizePercentage { get; set; } = 1;

    private ContainerService _containerService =
      ServiceLocator.Provider.GetRequiredService<ContainerService>();

    public override int Width => _containerService.CalculateWidthOfResizableContainer(this);

    public override int Height => _containerService.CalculateHeightOfResizableContainer(this);

    public override int X => _containerService.CalculateXOfResizableContainer(this);

    public override int Y => _containerService.CalculateYOfResizableContainer(this);

    public TilingWindow(IntPtr hwnd, Rectangle floatingPlacement) : base(hwnd, floatingPlacement)
    {
    }
  }
}
