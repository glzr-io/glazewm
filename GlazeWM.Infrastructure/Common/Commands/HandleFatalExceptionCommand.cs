using System;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Infrastructure.Common.Commands
{
  public class HandleFatalExceptionCommand : Command
  {
    public Exception Exception { get; }

    public HandleFatalExceptionCommand(Exception exception)
    {
      Exception = exception;
    }
  }
}
