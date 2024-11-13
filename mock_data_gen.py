from faker import Faker
import csv

fake = Faker()
rows_patient_info = []
rows_patient_relation = []
rows_patient_data = []

for i in range(20_000):  # Number of patients
    rows_patient_info.append([i,fake.first_name(), fake.last_name(), fake.phone_number(), fake.date_of_birth(), fake.address()])
    
    rows_patient_data.append([i,fake.date_this_year(), fake.sentence(), fake.text(max_nb_chars=500), fake.random_company_acronym()])

with open('patient_info.csv', 'w', newline='') as file:
    writer = csv.writer(file)
    writer.writerow(['patient_id','first_name', 'last_name', 'mobile_number', 'date_of_birth', 'address'])
    writer.writerows(rows_patient_info)

with open('patient_data.csv', 'w', newline='') as file:
    writer = csv.writer(file)
    writer.writerow(['patient_id','visit_date', 'diagnosis_overview', 'detailed_notes', 'medical_test_info'])

    writer.writerows(rows_patient_data)
