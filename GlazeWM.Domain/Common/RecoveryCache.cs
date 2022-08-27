using GlazeWM.Domain.Containers;

namespace GlazeWM.Domain.Common
{
  public class RecoveryCache
  {
    public string SessionId { get; init; }
    public Container ContainerTree { get; init; }

    public RecoveryCache(string sessionId, Container containerTree)
    {
      SessionId = sessionId;
      ContainerTree = containerTree;
    }
  }
}
