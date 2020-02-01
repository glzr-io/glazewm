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
        public static ServiceCollection ServiceCollection { get; private set; } = new ServiceCollection();

        /// <summary>
        ///  The main entry point for the application.
        /// </summary>
        [STAThread]
        static void Main()
        {
            Debug.WriteLine("Application started");

            ConfigureServices();

            var serviceProvider = ServiceCollection.BuildServiceProvider();

            var bus = serviceProvider.GetRequiredService<IBus>();

            bus.RegisterCommandHandler<AddMonitorHandler>();
            bus.RegisterEventHandler<MonitorAddedHandler>();

            var startup = serviceProvider.GetRequiredService<Startup>();
            startup.Init();
        }

        private static void ConfigureServices()
        {
            ServiceCollection.AddSingleton<IBus, Bus>();
            ServiceCollection.AddSingleton<AddMonitorHandler>();
            ServiceCollection.AddSingleton<MonitorAddedHandler>();
            ServiceCollection.AddSingleton<Startup>();
        }
    }
}
