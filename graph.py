import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

data = pd.read_csv("benchmark_data.csv")


#n_groups = data["test"].max()
#idx = np.arange(n_groups)

def make_avg_graph(data):
    df = pd.DataFrame(data)

    average_mnps = df.groupby('name')['m_nps'].mean().reset_index()

    plt.figure(figsize=(8, 4))
    bars = plt.bar(average_mnps['name'], average_mnps['m_nps'])

    plt.xlabel('Crate')
    plt.ylabel('Avg Millions Nodes/Sec\n(Higher is better)')
    plt.title('Efficiency of Rust Chess Move Generators')

    tick_positions = range(len(average_mnps['name']))
    tick_labels = average_mnps['name']
    plt.xticks(tick_positions, tick_labels, rotation=45)

    for bar in bars:
        yval = bar.get_height()
        plt.text(bar.get_x() + bar.get_width() / 2, yval, f"{yval:.2f}", 
                ha='center', va='bottom', fontsize=10)

    max_height = max(average_mnps['m_nps'])
    plt.ylim(0, max_height * 1.1)  # Set y-axis limit


    plt.tight_layout()  # Adjust layout to fit labels
    plt.show()


def make_barcode(data):
    df = pd.DataFrame(data)
    pivot_df = df.pivot(index='test', columns='name', values='m_nps')
    pivot_df.plot(kind='bar', figsize=(12, 6))

    plt.xlabel('Test #')
    plt.ylabel('Average NPS\n(higher = better)')
    plt.title('Move generation efficiency by position')
    plt.xticks(rotation=0) 
    plt.legend(title='Crate')  # Add legend

    plt.tight_layout()  # Adjust layout to fit labels
    plt.show()


# make_barcode(data)
make_avg_graph(data)
