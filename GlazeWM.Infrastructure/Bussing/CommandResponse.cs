namespace GlazeWM.Infrastructure.Bussing
{
  public class CommandResponse
  {
    public bool Success { get; private set; }
    public static readonly CommandResponse Ok = new() { Success = true };
    public static readonly CommandResponse Fail = new() { Success = false };

    public CommandResponse(bool success = false)
    {
      Success = success;
    }
  }
}
