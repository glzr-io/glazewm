using System;

namespace GlazeWM.Domain.Containers
{
  public sealed class RootContainer : Container
  {
    public override string Id { get; init; } = $"root_container/{new Guid()}";
  }
}
