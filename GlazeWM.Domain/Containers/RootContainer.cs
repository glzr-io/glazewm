using GlazeWM.Domain.Common;

namespace GlazeWM.Domain.Containers
{
  public sealed class RootContainer : Container
  {
    /// <inheritdoc />
    public override ContainerType Type { get; } = ContainerType.Root;
  }
}
