using System;

namespace LarsWM.Infrastructure.Bussing
{
  public class CommandResponse
  {
    public Boolean Success { get; private set; }
    public Guid AggregateId { get; private set; }
    public string Description { get; private set; }

    public static CommandResponse Ok = new CommandResponse { Success = true };

    public CommandResponse(Boolean success = false, Guid aggregateId = default)
    {
      Success = success;
      AggregateId = aggregateId;
      Description = String.Empty;
    }
  }
}
