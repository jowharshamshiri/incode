// InCode Test Debuggee - Threading Scenarios
// Multi-threading scenarios for Thread Management tools (F0041-F0045)

#include <iostream>
#include <thread>
#include <mutex>
#include <condition_variable>
#include <atomic>
#include <chrono>
#include <vector>
#include <queue>
#include <string>

// Global threading state variables for inspection
std::atomic<int> global_thread_counter{0};
std::atomic<bool> shutdown_requested{false};
std::mutex global_mutex;
std::condition_variable global_cv;
std::queue<int> shared_work_queue;

// Thread-local storage for testing
thread_local int thread_local_id = 0;
thread_local std::string thread_local_name;

// Worker thread function - creates predictable thread state
void worker_thread(int thread_id, const std::string& thread_name) {
    // Set thread-local variables
    thread_local_id = thread_id;
    thread_local_name = thread_name;
    
    std::cout << "Worker thread " << thread_id << " (" << thread_name << ") started" << std::endl;
    
    // Thread-specific variables for debugging inspection
    int local_work_count = 0;
    double local_processing_time = 0.0;
    bool local_active = true;
    
    while (!shutdown_requested && local_work_count < 10) {
        // Critical section with mutex - good for breakpoint testing
        {
            std::unique_lock<std::mutex> lock(global_mutex);
            
            // Wait for work or shutdown signal
            global_cv.wait(lock, []() { 
                return !shared_work_queue.empty() || shutdown_requested.load(); 
            });
            
            if (shutdown_requested.load()) {
                break;
            }
            
            // Process work item
            if (!shared_work_queue.empty()) {
                int work_item = shared_work_queue.front();
                shared_work_queue.pop();
                lock.unlock();
                
                // Simulate processing time
                auto start_time = std::chrono::high_resolution_clock::now();
                std::this_thread::sleep_for(std::chrono::milliseconds(100 + (thread_id * 10)));
                auto end_time = std::chrono::high_resolution_clock::now();
                
                local_processing_time += std::chrono::duration<double>(end_time - start_time).count();
                local_work_count++;
                
                std::cout << "Thread " << thread_id << " processed work item " << work_item 
                          << " (total processed: " << local_work_count << ")" << std::endl;
                
                // Update global counter
                global_thread_counter++;
            }
        }
    }
    
    local_active = false;
    std::cout << "Worker thread " << thread_id << " (" << thread_name << ") completed. "
              << "Work items processed: " << local_work_count 
              << ", Processing time: " << local_processing_time << "s" << std::endl;
}

// Producer thread - generates work items
void producer_thread() {
    thread_local_id = 999;
    thread_local_name = "Producer";
    
    std::cout << "Producer thread started" << std::endl;
    
    int work_item_id = 1;
    while (!shutdown_requested.load() && work_item_id <= 30) {
        {
            std::lock_guard<std::mutex> lock(global_mutex);
            shared_work_queue.push(work_item_id);
            std::cout << "Producer added work item " << work_item_id << std::endl;
        }
        
        // Notify one worker
        global_cv.notify_one();
        
        work_item_id++;
        std::this_thread::sleep_for(std::chrono::milliseconds(50));
    }
    
    std::cout << "Producer thread completed" << std::endl;
}

// Monitoring thread - provides thread state information
void monitor_thread() {
    thread_local_id = 998;
    thread_local_name = "Monitor";
    
    std::cout << "Monitor thread started" << std::endl;
    
    int monitor_iteration = 0;
    while (!shutdown_requested.load() && monitor_iteration < 20) {
        std::this_thread::sleep_for(std::chrono::milliseconds(250));
        
        {
            std::lock_guard<std::mutex> lock(global_mutex);
            std::cout << "Monitor: Queue size=" << shared_work_queue.size() 
                      << ", Global counter=" << global_thread_counter 
                      << ", Iteration=" << monitor_iteration << std::endl;
        }
        
        monitor_iteration++;
    }
    
    std::cout << "Monitor thread completed" << std::endl;
}

// Blocking thread - demonstrates blocked thread state
void blocking_thread() {
    thread_local_id = 997;
    thread_local_name = "Blocker";
    
    std::cout << "Blocking thread started - will wait indefinitely" << std::endl;
    
    std::unique_lock<std::mutex> lock(global_mutex);
    // This will block until shutdown is requested
    global_cv.wait(lock, []() { return shutdown_requested.load(); });
    
    std::cout << "Blocking thread unblocked and completed" << std::endl;
}

// CPU-intensive thread for testing different thread states
void cpu_intensive_thread(int thread_id) {
    thread_local_id = 800 + thread_id;
    thread_local_name = "CPU-Worker-" + std::to_string(thread_id);
    
    std::cout << "CPU-intensive thread " << thread_id << " started" << std::endl;
    
    volatile long long computation_result = 0;
    int iterations = 0;
    
    while (!shutdown_requested.load() && iterations < 1000000) {
        // CPU-intensive computation
        for (int i = 0; i < 1000; i++) {
            computation_result += i * thread_id;
        }
        iterations++;
        
        // Brief pause every 100k iterations to allow debugging
        if (iterations % 100000 == 0) {
            std::this_thread::sleep_for(std::chrono::microseconds(10));
            std::cout << "CPU thread " << thread_id << " iteration " << iterations 
                      << ", result=" << computation_result << std::endl;
        }
    }
    
    std::cout << "CPU-intensive thread " << thread_id << " completed after " 
              << iterations << " iterations" << std::endl;
}

// Main threading scenario orchestrator
void run_threading_scenarios() {
    std::cout << "Starting multi-threading scenarios..." << std::endl;
    
    // Reset global state
    global_thread_counter = 0;
    shutdown_requested.store(false);
    
    // Clear the work queue
    {
        std::lock_guard<std::mutex> lock(global_mutex);
        while (!shared_work_queue.empty()) {
            shared_work_queue.pop();
        }
    }
    
    // Create various types of threads for comprehensive testing
    std::vector<std::thread> threads;
    
    // Worker threads with different IDs and names
    threads.emplace_back(worker_thread, 1, "Worker-Alpha");
    threads.emplace_back(worker_thread, 2, "Worker-Beta");
    threads.emplace_back(worker_thread, 3, "Worker-Gamma");
    
    // Producer thread
    threads.emplace_back(producer_thread);
    
    // Monitor thread
    threads.emplace_back(monitor_thread);
    
    // Blocking thread (will demonstrate blocked state)
    threads.emplace_back(blocking_thread);
    
    // CPU-intensive threads
    threads.emplace_back(cpu_intensive_thread, 1);
    threads.emplace_back(cpu_intensive_thread, 2);
    
    std::cout << "Created " << threads.size() << " threads for testing" << std::endl;
    std::cout << "Threads are now running - good point for thread inspection tools" << std::endl;
    
    // Let threads run for a predictable amount of time
    std::this_thread::sleep_for(std::chrono::seconds(5));
    
    std::cout << "Initiating thread shutdown..." << std::endl;
    
    // Signal shutdown
    {
        std::lock_guard<std::mutex> lock(global_mutex);
        shutdown_requested.store(true);
    }
    
    // Notify all waiting threads
    global_cv.notify_all();
    
    // Join all threads
    for (auto& t : threads) {
        if (t.joinable()) {
            t.join();
        }
    }
    
    std::cout << "All threads completed. Final global counter: " << global_thread_counter << std::endl;
    std::cout << "Threading scenarios complete." << std::endl;
}