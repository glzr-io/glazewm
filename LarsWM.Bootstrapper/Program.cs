using LarsWM.Domain;
using LarsWM.Infrastructure;
using LarsWM.Infrastructure.Bussing;
using Microsoft.Extensions.DependencyInjection;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Threading.Tasks;
using System.Windows.Forms;

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

            ServiceLocator.Provider = serviceCollection.BuildServiceProvider();

            ServiceLocator.Provider.RegisterDomainHandlers();
        }
    }
}
