using LarsWM.Bar;
using LarsWM.Domain;
using LarsWM.Infrastructure;
using Microsoft.Extensions.DependencyInjection;
using System;
using System.Diagnostics;

namespace LarsWM.Bootstrapper
{
  static class Program
  {
    /// <summary>
    ///  The main entry point for the application.
    /// </summary>
    [STAThread]
    static void Main()
    {
      Debug.WriteLine("Application started");

      var serviceCollection = new ServiceCollection();
      serviceCollection.AddInfrastructureServices();
      serviceCollection.AddDomainServices();
      serviceCollection.AddBarServices();
      serviceCollection.AddSingleton<Startup>();

      ServiceLocator.Provider = serviceCollection.BuildServiceProvider();

      ServiceLocator.Provider.RegisterDomainHandlers();
      ServiceLocator.Provider.RegisterBarHandlers();

      var startup = ServiceLocator.Provider.GetRequiredService<Startup>();
      startup.Init();
    }
  }
}
