using System;

namespace GlazeWM.Infrastructure.Bussing
{
  public class CommandResponse
  {
    public bool Success { get; private set; }
    public static readonly CommandResponse Ok = new CommandResponse { Success = true };
    public static readonly CommandResponse Fail = new CommandResponse { Success = false };

    public CommandResponse(bool success = false)
    {
      Success = success;
    }
  }
}
