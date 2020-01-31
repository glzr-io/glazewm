using LarsWM.Core.Common.Services;
using Microsoft.Extensions.DependencyInjection;
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Windows.Forms;

namespace LarsWM.Core
{
    class Program
    {
        /// <summary>
        ///  The main entry point for the application.
        /// </summary>
        [STAThread]
        static void Main()
        {
            Debug.WriteLine("Application started");

            var serviceCollection = new ServiceCollection();
            ConfigureServices(serviceCollection);

            var serviceProvider = serviceCollection.BuildServiceProvider();

            var startup = serviceProvider.GetRequiredService<Startup>();
            startup.Init();

            // TODO: Read config file and initialise UserConfig class with its values
            // TODO: Register windows hooks
            // TODO: Create a workspace and assign a workspace to each connected display
            // TODO: Force initial layout
        }

        private static void ConfigureServices(ServiceCollection serviceCollection)
        {
            serviceCollection.AddSingleton<IBus, Bus>();
            serviceCollection.AddSingleton<Startup>();
        }
    }
}
