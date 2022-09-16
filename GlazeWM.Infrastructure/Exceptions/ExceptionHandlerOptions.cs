using System;

namespace GlazeWM.Infrastructure.Exceptions
{
  public class ExceptionHandlerOptions
  {
    public string ErrorLogPath { get; set; }
    public Func<Exception, string> ErrorLogMessageDelegate { get; set; }
  }
}
