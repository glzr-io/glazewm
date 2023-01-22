using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class ResizeContainerCommand : Command
  {
    public Container ContainerToResize { get; }
    public double ResizePercentage { get; }

    public ResizeContainerCommand(Container containerToResize, double resizePercentage)
    {
      ContainerToResize = containerToResize;
      ResizePercentage = resizePercentage;
    }
  }
}
