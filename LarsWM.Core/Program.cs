using LarsWM.Core.Common.Services;
using LarsWM.Core.Monitors.CommandHandlers;
using LarsWM.Core.Monitors.EventHandler;
using Microsoft.Extensions.DependencyInjection;
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Windows.Forms;

namespace LarsWM.Core
{
    class Program
    {
        public static IServiceProvider ServiceProvider { get; private set; }

        /// <summary>
        ///  The main entry point for the application.
        /// </summary>
        [STAThread]
        static void Main()
        {
            Debug.WriteLine("Application started");

            var serviceCollection = CreateServiceCollection();            

            ServiceProvider = serviceCollection.BuildServiceProvider();

            var bus = ServiceProvider.GetRequiredService<IBus>();
            bus.RegisterCommandHandler<AddMonitorHandler>();
            bus.RegisterEventHandler<MonitorAddedHandler>();

            var startup = ServiceProvider.GetRequiredService<Startup>();
            startup.Init();
        }

        private static ServiceCollection CreateServiceCollection()
        {
            var serviceCollection = new ServiceCollection();

            serviceCollection.AddSingleton<IBus, Bus>();
            serviceCollection.AddSingleton<AppState>();
            serviceCollection.AddSingleton<AddMonitorHandler>();
            serviceCollection.AddSingleton<MonitorAddedHandler>();
            serviceCollection.AddSingleton<Startup>();

            return serviceCollection;
        }
    }
}
