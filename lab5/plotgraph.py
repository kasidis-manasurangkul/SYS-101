import matplotlib.pyplot as plt
import pandas as pd

# Load the data from the file
df = pd.read_csv('data.txt', sep=',', names=['Type', 'Parameter', 'Time'])

# Separate the data based on the 'Type' column
df_threads = df[df['Type'].isin(['Traditional', 'Scalable'])]
df_approx = df[df['Type'] == 'ApproximationFactor']

# Check if the dataframes are not empty
if not df_threads.empty:
    # Plot 1: Threads vs. Time
    plt.figure(figsize=(8, 6))
    for t in df_threads['Type'].unique():
        subset = df_threads[df_threads['Type'] == t]
        plt.plot(subset['Parameter'], subset['Time'], label=t, marker='o')

    plt.xlabel('Number of Threads')
    plt.ylabel('Runtime (milliseconds)')
    plt.title('Runtime Comparison with Varying Number of Threads')
    plt.legend()
    plt.grid(True)
    plt.savefig('threads_vs_time.png')
else:
    print("No data available for Threads vs. Time plot.")

if not df_approx.empty:
    # Plot 2: Approximation Factor vs. Time
    plt.figure(figsize=(8, 6))
    plt.plot(df_approx['Parameter'], df_approx['Time'], label='Approximation Factor', marker='o')
    plt.xlabel('Approximation Factor (S)')
    plt.ylabel('Runtime (milliseconds)')
    plt.title('Runtime Comparison with Varying Approximation Factor')
    plt.legend()
    plt.grid(True)
    plt.xscale('log', base=2)
    plt.xticks([2**i for i in range(int(np.log2(df_approx['Parameter'].min())), 
                                     int(np.log2(df_approx['Parameter'].max()))+1)], 
               [str(2**i) for i in range(int(np.log2(df_approx['Parameter'].min())), 
                                          int(np.log2(df_approx['Parameter'].max()))+1)])
    plt.savefig('approx_factor_vs_time.png')
else:
    print("No data available for Approximation Factor vs. Time plot.")
