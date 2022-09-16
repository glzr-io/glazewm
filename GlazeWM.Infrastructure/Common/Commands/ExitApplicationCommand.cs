using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Commands
{
  public class ExitApplicationCommand : Command
  {
    public bool WithErrorCode { get; }

    public ExitApplicationCommand(bool withErrorCode)
    {
      WithErrorCode = withErrorCode;
    }
  }
}
