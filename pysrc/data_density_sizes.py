import json
import numpy as np


class Data:
    def __init__(self, data_dir: str, file: str):
        offset = 901
        num_sample_files = 1
        # offset = 1
        # num_sample_files = 20

        with open(f"{data_dir}/{file}{offset}.json") as f:
            data = json.load(f)

        self.densities = data["densities"]
        self.sizes = data["sizes"]
        self.density_len = len(self.densities)
        self.size_len = len(self.sizes)

        num_total_samples = 0

        self.simplicial = np.array(
            np.tile(0, (self.size_len, self.density_len)), dtype=float
        )
        self.before_simplicial = np.array(
            np.tile(0, (self.size_len, self.density_len)), dtype=float
        )
        self.claw_free = np.array(
            np.tile(0, (self.size_len, self.density_len)), dtype=float
        )
        self.before_claw_free = np.array(
            np.tile(0, (self.size_len, self.density_len)), dtype=float
        )
        self.collapsed = np.array(
            np.tile(0, (self.size_len, self.density_len)), dtype=float
        )

        for i in range(offset, num_sample_files + offset):
            try:
                with open(f"{data_dir}/{file}{i}.json") as f:
                    data = json.load(f)
            except FileNotFoundError:
                print(f"File {file}{i} not found")
                continue
            num_samples = data["num_samples"]
            num_total_samples += num_samples
            self.simplicial += num_samples * np.array(data["after_simplicial"])
            self.before_simplicial += num_samples * np.array(data["before_simplicial"])
            self.claw_free += num_samples * np.array(data["after_claw_free"])
            self.before_claw_free += num_samples * np.array(data["before_claw_free"])
            self.collapsed += num_samples * np.array(data["collapsed"])
        print(num_total_samples)
        self.simplicial /= num_total_samples
        self.before_simplicial /= num_total_samples
        self.delta_simplicial = (self.simplicial - self.before_simplicial) * 100
        self.claw_free /= num_total_samples
        self.before_claw_free /= num_total_samples
        self.delta_claw_free = (self.claw_free - self.before_claw_free) * 100
        self.collapsed *= 100.0 / num_total_samples
