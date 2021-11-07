using System;

namespace GlazeWM.Infrastructure.Bussing
{
  public class CommandResponse
  {
    public Boolean Success { get; private set; }
    public static CommandResponse Ok = new CommandResponse { Success = true };
    public static CommandResponse Fail = new CommandResponse { Success = false };

    public CommandResponse(Boolean success = false)
    {
      Success = success;
    }
  }
}
