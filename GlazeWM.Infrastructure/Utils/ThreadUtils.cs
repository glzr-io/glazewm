using System;
using System.Threading;

namespace GlazeWM.Infrastructure.Utils
{
  public static class ThreadUtils
  {
    public static void CreateSTA(string threadName, Action threadAction)
    {
      var thread = new Thread(() => threadAction())
      {
        Name = threadName
      };

      thread.SetApartmentState(ApartmentState.STA);
      thread.Start();
    }

    public static void Create(string threadName, Action threadAction)
    {
      var thread = new Thread(() => threadAction())
      {
        Name = threadName
      };

      thread.Start();
    }
  }
}
